use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{
    Data, DeriveInput, Error, Expr, ExprLit, Field, Fields, Ident, Lit, LitStr, Path, Result, Type,
    parse_macro_input,
};

#[proc_macro_derive(Entity, attributes(orm))]
pub fn derive_entity(input: TokenStream) -> TokenStream {
    match derive_entity_impl(parse_macro_input!(input as DeriveInput)) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.to_compile_error().into(),
    }
}

#[proc_macro_derive(DbContext, attributes(orm))]
pub fn derive_db_context(_input: TokenStream) -> TokenStream {
    TokenStream::new()
}

#[proc_macro_derive(Insertable, attributes(orm))]
pub fn derive_insertable(input: TokenStream) -> TokenStream {
    match derive_insertable_impl(parse_macro_input!(input as DeriveInput)) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.to_compile_error().into(),
    }
}

#[proc_macro_derive(Changeset, attributes(orm))]
pub fn derive_changeset(input: TokenStream) -> TokenStream {
    match derive_changeset_impl(parse_macro_input!(input as DeriveInput)) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.to_compile_error().into(),
    }
}

fn derive_entity_impl(input: DeriveInput) -> Result<TokenStream2> {
    let ident = input.ident;
    let entity_config = parse_entity_config(&input.attrs)?;
    let fields = match input.data {
        Data::Struct(data) => match data.fields {
            Fields::Named(fields) => fields.named,
            _ => {
                return Err(Error::new_spanned(
                    ident,
                    "Entity solo soporta structs con campos nombrados",
                ));
            }
        },
        _ => return Err(Error::new_spanned(ident, "Entity solo soporta structs")),
    };

    let schema = entity_config
        .schema
        .unwrap_or_else(|| LitStr::new("dbo", Span::call_site()));
    let table = entity_config
        .table
        .unwrap_or_else(|| LitStr::new(&default_table_name(&ident), ident.span()));
    let rust_name = LitStr::new(&ident.to_string(), ident.span());

    let mut columns = Vec::new();
    let mut column_symbols = Vec::new();
    let mut primary_key_columns = Vec::new();
    let mut indexes = Vec::new();

    let has_explicit_primary_key = has_explicit_primary_key(&fields)?;

    for field in fields.iter() {
        let field_ident = field
            .ident
            .as_ref()
            .ok_or_else(|| Error::new_spanned(field, "Entity requiere campos nombrados"))?;
        let config = parse_field_config(field)?;
        let rust_field = LitStr::new(&field_ident.to_string(), field_ident.span());
        let column_name = config
            .column
            .unwrap_or_else(|| LitStr::new(&field_ident.to_string(), field_ident.span()));
        let type_info = analyze_type(&field.ty)?;

        let primary_key = config.primary_key
            || (field_ident == &Ident::new("id", field_ident.span()) && !has_explicit_primary_key);
        if primary_key {
            primary_key_columns.push(column_name.clone());
        }

        let sql_type = match config.sql_type {
            Some(sql_type) => sql_type_from_string(&sql_type),
            None => infer_sql_type(&type_info, config.rowversion, &field.ty)?,
        };

        if config.identity && !type_info.is_integer {
            return Err(Error::new_spanned(
                &field.ty,
                "identity solo se soporta sobre tipos enteros",
            ));
        }

        if config.rowversion && !type_info.is_vec_u8 {
            return Err(Error::new_spanned(&field.ty, "rowversion requiere Vec<u8>"));
        }

        let nullable = config.nullable || type_info.nullable;
        let identity = if config.identity {
            let seed = config.identity_seed.unwrap_or(1);
            let increment = config.identity_increment.unwrap_or(1);
            quote! {
                Some(::mssql_orm::core::IdentityMetadata::new(#seed, #increment))
            }
        } else {
            quote! { None }
        };

        let max_length = config
            .length
            .or_else(|| type_info.default_max_length.filter(|_| !config.rowversion));
        let precision = config.precision.or(type_info.default_precision);
        let scale = config.scale.or(type_info.default_scale);
        let default_sql = option_lit_str(config.default_sql);
        let has_computed_sql = config.computed_sql.is_some();
        let computed_sql = option_lit_str(config.computed_sql);
        let max_length = option_number(max_length);
        let precision = option_number(precision);
        let scale = option_number(scale);
        let rowversion = config.rowversion;
        let insertable = !config.identity && !rowversion && !has_computed_sql;
        let updatable = !primary_key && !rowversion && !has_computed_sql;

        columns.push(quote! {
            ::mssql_orm::core::ColumnMetadata {
                rust_field: #rust_field,
                column_name: #column_name,
                sql_type: #sql_type,
                nullable: #nullable,
                primary_key: #primary_key,
                identity: #identity,
                default_sql: #default_sql,
                computed_sql: #computed_sql,
                rowversion: #rowversion,
                insertable: #insertable,
                updatable: #updatable,
                max_length: #max_length,
                precision: #precision,
                scale: #scale,
            }
        });

        column_symbols.push(quote! {
            pub const #field_ident: ::mssql_orm::core::EntityColumn<#ident> =
                ::mssql_orm::core::EntityColumn::new(#rust_field, #column_name);
        });

        for index in config.indexes {
            let index_name = index.name.unwrap_or_else(|| {
                generated_index_name(
                    if index.unique { "ux" } else { "ix" },
                    table.value().as_str(),
                    column_name.value().as_str(),
                    field_ident.span(),
                )
            });
            let unique = index.unique;

            indexes.push(quote! {
                ::mssql_orm::core::IndexMetadata {
                    name: #index_name,
                    columns: &[::mssql_orm::core::IndexColumnMetadata::asc(#column_name)],
                    unique: #unique,
                }
            });
        }
    }

    if primary_key_columns.is_empty() {
        return Err(Error::new_spanned(
            ident,
            "Entity requiere al menos una primary key",
        ));
    }

    let metadata_ident = Ident::new(
        &format!("__MSSQL_ORM_ENTITY_METADATA_FOR_{}", ident),
        Span::call_site(),
    );

    Ok(quote! {
        static #metadata_ident: ::mssql_orm::core::EntityMetadata = ::mssql_orm::core::EntityMetadata {
            rust_name: #rust_name,
            schema: #schema,
            table: #table,
            columns: &[#(#columns),*],
            primary_key: ::mssql_orm::core::PrimaryKeyMetadata::new(
                None,
                &[#(#primary_key_columns),*],
            ),
            indexes: &[#(#indexes),*],
            foreign_keys: &[],
        };

        #[allow(non_upper_case_globals)]
        impl #ident {
            #(#column_symbols)*
        }

        impl ::mssql_orm::core::Entity for #ident {
            fn metadata() -> &'static ::mssql_orm::core::EntityMetadata {
                &#metadata_ident
            }
        }
    })
}

fn derive_insertable_impl(input: DeriveInput) -> Result<TokenStream2> {
    let ident = input.ident;
    let model_config = parse_persistence_model_config(&input.attrs, "Insertable")?;
    let entity = model_config
        .entity
        .as_ref()
        .expect("validated persistence model must include entity");
    let fields = extract_named_fields(&ident, input.data, "Insertable")?;

    let values = fields
        .iter()
        .map(|field| {
            let field_ident = field
                .ident
                .as_ref()
                .ok_or_else(|| Error::new_spanned(field, "Insertable requiere campos nombrados"))?;
            let field_config = parse_persistence_field_config(field, "Insertable")?;
            let field_ty = &field.ty;
            let column_name =
                persistence_column_name_expr(entity, field_ident, field_config.column.as_ref());

            Ok(quote! {
                ::mssql_orm::core::ColumnValue::new(
                    #column_name,
                    <#field_ty as ::mssql_orm::core::SqlTypeMapping>::to_sql_value(
                        ::core::clone::Clone::clone(&self.#field_ident)
                    ),
                )
            })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(quote! {
        impl ::mssql_orm::core::Insertable<#entity> for #ident {
            fn values(&self) -> ::std::vec::Vec<::mssql_orm::core::ColumnValue> {
                ::std::vec![#(#values),*]
            }
        }
    })
}

fn derive_changeset_impl(input: DeriveInput) -> Result<TokenStream2> {
    let ident = input.ident;
    let model_config = parse_persistence_model_config(&input.attrs, "Changeset")?;
    let entity = model_config
        .entity
        .as_ref()
        .expect("validated persistence model must include entity");
    let fields = extract_named_fields(&ident, input.data, "Changeset")?;

    let changes = fields
        .iter()
        .map(|field| {
            let field_ident = field
                .ident
                .as_ref()
                .ok_or_else(|| Error::new_spanned(field, "Changeset requiere campos nombrados"))?;
            let field_config = parse_persistence_field_config(field, "Changeset")?;
            let inner_ty = option_inner_type(&field.ty).ok_or_else(|| {
                Error::new_spanned(
                    &field.ty,
                    "Changeset requiere Option<T> en cada campo para distinguir campos omitidos",
                )
            })?;
            let column_name =
                persistence_column_name_expr(entity, field_ident, field_config.column.as_ref());

            Ok(quote! {
                if let ::core::option::Option::Some(value) = &self.#field_ident {
                    changes.push(::mssql_orm::core::ColumnValue::new(
                        #column_name,
                        <#inner_ty as ::mssql_orm::core::SqlTypeMapping>::to_sql_value(
                            ::core::clone::Clone::clone(value)
                        ),
                    ));
                }
            })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(quote! {
        impl ::mssql_orm::core::Changeset<#entity> for #ident {
            fn changes(&self) -> ::std::vec::Vec<::mssql_orm::core::ColumnValue> {
                let mut changes = ::std::vec::Vec::new();
                #(#changes)*
                changes
            }
        }
    })
}

fn extract_named_fields(
    ident: &Ident,
    data: Data,
    derive_name: &str,
) -> Result<syn::punctuated::Punctuated<Field, syn::token::Comma>> {
    match data {
        Data::Struct(data) => match data.fields {
            Fields::Named(fields) => Ok(fields.named),
            _ => Err(Error::new_spanned(
                ident,
                format!("{derive_name} solo soporta structs con campos nombrados"),
            )),
        },
        _ => Err(Error::new_spanned(
            ident,
            format!("{derive_name} solo soporta structs"),
        )),
    }
}

fn has_explicit_primary_key(
    fields: &syn::punctuated::Punctuated<Field, syn::token::Comma>,
) -> Result<bool> {
    for field in fields {
        if parse_field_config(field)?.primary_key {
            return Ok(true);
        }
    }
    Ok(false)
}

fn parse_persistence_model_config(
    attrs: &[syn::Attribute],
    derive_name: &str,
) -> Result<PersistenceModelConfig> {
    let mut config = PersistenceModelConfig::default();

    for attr in attrs {
        if !attr.path().is_ident("orm") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("entity") {
                config.entity = Some(meta.value()?.parse()?);
            } else {
                return Err(meta.error(format!(
                    "atributo orm no soportado a nivel de {derive_name}"
                )));
            }

            Ok(())
        })?;
    }

    let Some(entity) = config.entity else {
        return Err(Error::new(
            Span::call_site(),
            format!("{derive_name} requiere #[orm(entity = MiEntidad)]"),
        ));
    };

    Ok(PersistenceModelConfig {
        entity: Some(entity),
    })
}

fn parse_entity_config(attrs: &[syn::Attribute]) -> Result<EntityConfig> {
    let mut config = EntityConfig::default();

    for attr in attrs {
        if !attr.path().is_ident("orm") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("table") {
                config.table = Some(parse_lit_str(meta.value()?.parse()?)?);
            } else if meta.path.is_ident("schema") {
                config.schema = Some(parse_lit_str(meta.value()?.parse()?)?);
            } else {
                return Err(meta.error("atributo orm no soportado a nivel de entidad"));
            }

            Ok(())
        })?;
    }

    Ok(config)
}

fn parse_persistence_field_config(
    field: &Field,
    derive_name: &str,
) -> Result<PersistenceFieldConfig> {
    let mut config = PersistenceFieldConfig::default();

    for attr in &field.attrs {
        if !attr.path().is_ident("orm") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("column") {
                config.column = Some(parse_lit_str(meta.value()?.parse()?)?);
            } else {
                return Err(meta.error(format!(
                    "atributo orm no soportado en campos de {derive_name}"
                )));
            }

            Ok(())
        })?;
    }

    Ok(config)
}

fn parse_field_config(field: &Field) -> Result<FieldConfig> {
    let mut config = FieldConfig::default();

    for attr in &field.attrs {
        if !attr.path().is_ident("orm") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("column") {
                config.column = Some(parse_lit_str(meta.value()?.parse()?)?);
            } else if meta.path.is_ident("primary_key") {
                config.primary_key = true;
            } else if meta.path.is_ident("identity") {
                config.identity = true;
                if !meta.input.is_empty() {
                    meta.parse_nested_meta(|nested| {
                        if nested.path.is_ident("seed") {
                            config.identity_seed = Some(parse_i64_expr(nested.value()?.parse()?)?);
                        } else if nested.path.is_ident("increment") {
                            config.identity_increment =
                                Some(parse_i64_expr(nested.value()?.parse()?)?);
                        } else {
                            return Err(nested.error("identity solo soporta seed e increment"));
                        }

                        Ok(())
                    })?;
                }
            } else if meta.path.is_ident("length") {
                config.length = Some(parse_u32_expr(meta.value()?.parse()?)?);
            } else if meta.path.is_ident("nullable") {
                config.nullable = true;
            } else if meta.path.is_ident("default_sql") {
                config.default_sql = Some(parse_lit_str(meta.value()?.parse()?)?);
            } else if meta.path.is_ident("index") {
                let mut index = IndexConfig::default();
                meta.parse_nested_meta(|nested| {
                    if nested.path.is_ident("name") {
                        index.name = Some(parse_lit_str(nested.value()?.parse()?)?);
                    } else {
                        return Err(nested.error("index solo soporta name"));
                    }

                    Ok(())
                })?;
                config.indexes.push(index);
            } else if meta.path.is_ident("unique") {
                config.indexes.push(IndexConfig {
                    unique: true,
                    ..IndexConfig::default()
                });
            } else if meta.path.is_ident("sql_type") {
                config.sql_type = Some(parse_lit_str(meta.value()?.parse()?)?);
            } else if meta.path.is_ident("precision") {
                config.precision = Some(parse_u8_expr(meta.value()?.parse()?)?);
            } else if meta.path.is_ident("scale") {
                config.scale = Some(parse_u8_expr(meta.value()?.parse()?)?);
            } else if meta.path.is_ident("computed_sql") {
                config.computed_sql = Some(parse_lit_str(meta.value()?.parse()?)?);
            } else if meta.path.is_ident("rowversion") {
                config.rowversion = true;
            } else {
                return Err(meta.error("atributo orm no soportado a nivel de campo"));
            }

            Ok(())
        })?;
    }

    Ok(config)
}

fn parse_lit_str(expr: Expr) -> Result<LitStr> {
    match expr {
        Expr::Lit(ExprLit {
            lit: Lit::Str(value),
            ..
        }) => Ok(value),
        other => Err(Error::new_spanned(other, "se esperaba un string literal")),
    }
}

fn parse_u32_expr(expr: Expr) -> Result<u32> {
    parse_int::<u32>(expr, "se esperaba un entero u32")
}

fn parse_u8_expr(expr: Expr) -> Result<u8> {
    parse_int::<u8>(expr, "se esperaba un entero u8")
}

fn parse_i64_expr(expr: Expr) -> Result<i64> {
    parse_int::<i64>(expr, "se esperaba un entero i64")
}

fn parse_int<T>(expr: Expr, message: &str) -> Result<T>
where
    T: std::str::FromStr,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    match expr {
        Expr::Lit(ExprLit {
            lit: Lit::Int(value),
            ..
        }) => value
            .base10_parse::<T>()
            .map_err(|_| Error::new_spanned(value, message)),
        other => Err(Error::new_spanned(other, message)),
    }
}

fn option_lit_str(value: Option<LitStr>) -> TokenStream2 {
    match value {
        Some(value) => quote! { Some(#value) },
        None => quote! { None },
    }
}

fn persistence_column_name_expr(
    entity: &Type,
    field_ident: &Ident,
    explicit_column: Option<&LitStr>,
) -> TokenStream2 {
    let field_name = LitStr::new(&field_ident.to_string(), field_ident.span());

    match explicit_column {
        Some(column_name) => {
            let error = LitStr::new(
                &format!(
                    "la columna '{}' no existe en la metadata de la entidad de destino",
                    column_name.value()
                ),
                column_name.span(),
            );

            quote! {{
                <#entity as ::mssql_orm::core::Entity>::metadata()
                    .column(#column_name)
                    .expect(#error)
                    .column_name
            }}
        }
        None => {
            let error = LitStr::new(
                &format!(
                    "el campo '{}' no existe en la metadata de la entidad de destino",
                    field_ident
                ),
                field_ident.span(),
            );

            quote! {{
                <#entity as ::mssql_orm::core::Entity>::metadata()
                    .field(#field_name)
                    .expect(#error)
                    .column_name
            }}
        }
    }
}

fn option_number<T>(value: Option<T>) -> TokenStream2
where
    T: quote::ToTokens,
{
    match value {
        Some(value) => quote! { Some(#value) },
        None => quote! { None },
    }
}

fn infer_sql_type(type_info: &TypeInfo, rowversion: bool, ty: &Type) -> Result<TokenStream2> {
    if rowversion {
        return Ok(quote! { ::mssql_orm::core::SqlServerType::RowVersion });
    }

    let token = match type_info.kind {
        TypeKind::I64 => quote! { ::mssql_orm::core::SqlServerType::BigInt },
        TypeKind::I32 => quote! { ::mssql_orm::core::SqlServerType::Int },
        TypeKind::I16 => quote! { ::mssql_orm::core::SqlServerType::SmallInt },
        TypeKind::U8 => quote! { ::mssql_orm::core::SqlServerType::TinyInt },
        TypeKind::Bool => quote! { ::mssql_orm::core::SqlServerType::Bit },
        TypeKind::String => quote! { ::mssql_orm::core::SqlServerType::NVarChar },
        TypeKind::VecU8 => quote! { ::mssql_orm::core::SqlServerType::VarBinary },
        TypeKind::Uuid => quote! { ::mssql_orm::core::SqlServerType::UniqueIdentifier },
        TypeKind::NaiveDateTime => quote! { ::mssql_orm::core::SqlServerType::DateTime2 },
        TypeKind::NaiveDate => quote! { ::mssql_orm::core::SqlServerType::Date },
        TypeKind::Decimal => quote! { ::mssql_orm::core::SqlServerType::Decimal },
        TypeKind::Float => quote! { ::mssql_orm::core::SqlServerType::Float },
        TypeKind::Unknown => {
            return Err(Error::new_spanned(
                ty,
                "tipo Rust no soportado todavía para derive(Entity)",
            ));
        }
    };

    Ok(token)
}

fn sql_type_from_string(value: &LitStr) -> TokenStream2 {
    let normalized = value.value().to_ascii_lowercase();

    if normalized.starts_with("bigint") {
        quote! { ::mssql_orm::core::SqlServerType::BigInt }
    } else if normalized == "int" {
        quote! { ::mssql_orm::core::SqlServerType::Int }
    } else if normalized.starts_with("smallint") {
        quote! { ::mssql_orm::core::SqlServerType::SmallInt }
    } else if normalized.starts_with("tinyint") {
        quote! { ::mssql_orm::core::SqlServerType::TinyInt }
    } else if normalized.starts_with("bit") {
        quote! { ::mssql_orm::core::SqlServerType::Bit }
    } else if normalized.starts_with("uniqueidentifier") {
        quote! { ::mssql_orm::core::SqlServerType::UniqueIdentifier }
    } else if normalized.starts_with("date") && !normalized.starts_with("datetime2") {
        quote! { ::mssql_orm::core::SqlServerType::Date }
    } else if normalized.starts_with("datetime2") {
        quote! { ::mssql_orm::core::SqlServerType::DateTime2 }
    } else if normalized.starts_with("decimal") {
        quote! { ::mssql_orm::core::SqlServerType::Decimal }
    } else if normalized.starts_with("float") {
        quote! { ::mssql_orm::core::SqlServerType::Float }
    } else if normalized.starts_with("money") {
        quote! { ::mssql_orm::core::SqlServerType::Money }
    } else if normalized.starts_with("nvarchar") {
        quote! { ::mssql_orm::core::SqlServerType::NVarChar }
    } else if normalized.starts_with("varbinary") {
        quote! { ::mssql_orm::core::SqlServerType::VarBinary }
    } else if normalized.starts_with("rowversion") {
        quote! { ::mssql_orm::core::SqlServerType::RowVersion }
    } else {
        quote! { ::mssql_orm::core::SqlServerType::Custom(#value) }
    }
}

fn analyze_type(ty: &Type) -> Result<TypeInfo> {
    let nullable = option_inner_type(ty).is_some();
    let effective = option_inner_type(ty).unwrap_or(ty);
    let kind = classify_type(effective)?;

    Ok(TypeInfo {
        nullable,
        is_integer: matches!(
            kind,
            TypeKind::I64 | TypeKind::I32 | TypeKind::I16 | TypeKind::U8
        ),
        is_vec_u8: matches!(kind, TypeKind::VecU8),
        default_max_length: matches!(kind, TypeKind::String).then_some(255),
        default_precision: matches!(kind, TypeKind::Decimal).then_some(18),
        default_scale: matches!(kind, TypeKind::Decimal).then_some(2),
        kind,
    })
}

fn classify_type(ty: &Type) -> Result<TypeKind> {
    match ty {
        Type::Path(type_path) => {
            let segment = type_path
                .path
                .segments
                .last()
                .ok_or_else(|| Error::new_spanned(type_path, "tipo inválido"))?;

            let ident = segment.ident.to_string();
            let kind = match ident.as_str() {
                "i64" => TypeKind::I64,
                "i32" => TypeKind::I32,
                "i16" => TypeKind::I16,
                "u8" => TypeKind::U8,
                "bool" => TypeKind::Bool,
                "String" => TypeKind::String,
                "Uuid" => TypeKind::Uuid,
                "NaiveDateTime" => TypeKind::NaiveDateTime,
                "NaiveDate" => TypeKind::NaiveDate,
                "Decimal" => TypeKind::Decimal,
                "f32" | "f64" => TypeKind::Float,
                "Vec" if type_path_is_vec_u8(&type_path.path) => TypeKind::VecU8,
                _ => TypeKind::Unknown,
            };

            Ok(kind)
        }
        _ => Ok(TypeKind::Unknown),
    }
}

fn option_inner_type(ty: &Type) -> Option<&Type> {
    let Type::Path(type_path) = ty else {
        return None;
    };

    let segment = type_path.path.segments.last()?;
    if segment.ident != "Option" {
        return None;
    }

    let syn::PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return None;
    };

    let syn::GenericArgument::Type(inner) = arguments.args.first()? else {
        return None;
    };

    Some(inner)
}

fn type_path_is_vec_u8(path: &Path) -> bool {
    let Some(segment) = path.segments.last() else {
        return false;
    };

    if segment.ident != "Vec" {
        return false;
    }

    let syn::PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return false;
    };

    let Some(syn::GenericArgument::Type(Type::Path(inner_path))) = arguments.args.first() else {
        return false;
    };

    inner_path.path.is_ident("u8")
}

fn default_table_name(ident: &Ident) -> String {
    pluralize(&to_snake_case(&ident.to_string()))
}

fn to_snake_case(value: &str) -> String {
    let mut output = String::with_capacity(value.len());

    for (index, ch) in value.chars().enumerate() {
        if ch.is_uppercase() {
            if index > 0 {
                output.push('_');
            }

            for lower in ch.to_lowercase() {
                output.push(lower);
            }
        } else {
            output.push(ch);
        }
    }

    output
}

fn pluralize(value: &str) -> String {
    if ends_with_consonant_y(value) {
        let stem = &value[..value.len() - 1];
        format!("{stem}ies")
    } else if value.ends_with('s')
        || value.ends_with('x')
        || value.ends_with('z')
        || value.ends_with("ch")
        || value.ends_with("sh")
    {
        format!("{value}es")
    } else {
        format!("{value}s")
    }
}

fn ends_with_consonant_y(value: &str) -> bool {
    let mut chars = value.chars().rev();
    let Some(last) = chars.next() else {
        return false;
    };
    let Some(previous) = chars.next() else {
        return false;
    };

    last == 'y' && !matches!(previous, 'a' | 'e' | 'i' | 'o' | 'u')
}

fn generated_index_name(prefix: &str, table: &str, column: &str, span: Span) -> LitStr {
    LitStr::new(&format!("{prefix}_{table}_{column}"), span)
}

#[derive(Default)]
struct EntityConfig {
    table: Option<LitStr>,
    schema: Option<LitStr>,
}

#[derive(Default)]
struct PersistenceModelConfig {
    entity: Option<Type>,
}

#[derive(Default)]
struct PersistenceFieldConfig {
    column: Option<LitStr>,
}

#[derive(Default)]
struct FieldConfig {
    column: Option<LitStr>,
    primary_key: bool,
    identity: bool,
    identity_seed: Option<i64>,
    identity_increment: Option<i64>,
    nullable: bool,
    length: Option<u32>,
    default_sql: Option<LitStr>,
    computed_sql: Option<LitStr>,
    rowversion: bool,
    sql_type: Option<LitStr>,
    precision: Option<u8>,
    scale: Option<u8>,
    indexes: Vec<IndexConfig>,
}

#[derive(Default)]
struct IndexConfig {
    name: Option<LitStr>,
    unique: bool,
}

struct TypeInfo {
    nullable: bool,
    is_integer: bool,
    is_vec_u8: bool,
    default_max_length: Option<u32>,
    default_precision: Option<u8>,
    default_scale: Option<u8>,
    kind: TypeKind,
}

enum TypeKind {
    I64,
    I32,
    I16,
    U8,
    Bool,
    String,
    VecU8,
    Uuid,
    NaiveDateTime,
    NaiveDate,
    Decimal,
    Float,
    Unknown,
}
