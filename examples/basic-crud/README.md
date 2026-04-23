# basic-crud

Ejemplo funcional mínimo de CRUD usando la API pública de `mssql-orm`.

## Requisitos

- SQL Server accesible
- una connection string válida en `DATABASE_URL`

Ejemplo:

```bash
export DATABASE_URL='Server=localhost;Database=tempdb;User Id=SA;Password=Ea.930318;TrustServerCertificate=True;Encrypt=False;Connection Timeout=30;MultipleActiveResultSets=true;'
```

## Ejecutar

```bash
cargo run --manifest-path examples/basic-crud/Cargo.toml
```

## Qué hace

1. Crea o recrea `dbo.basic_crud_users`.
2. Inserta un usuario.
3. Lo busca por id.
4. Ejecuta `count`, `all` y `query_with(...).first()`.
5. Actualiza el usuario.
6. Lo elimina.
7. Limpia la tabla al final.
