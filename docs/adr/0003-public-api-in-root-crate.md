# ADR 0003: API pública concentrada en `mssql-orm`

## Estado

Aprobado

## Contexto

El workspace interno usa varias crates para separar responsabilidades, pero el consumidor del ORM necesita una experiencia de integración simple y estable. Exponer cada crate interna como puerta de entrada aumentaría el acoplamiento del usuario con detalles internos que pueden evolucionar.

## Decisión

La API pública soportada se concentra en la crate `mssql-orm`, que reexporta la superficie aprobada para consumo externo.

## Consecuencias

- Los consumidores dependen principalmente de `mssql-orm`.
- Los cambios internos entre crates pueden manejarse con menor fricción.
- Los derives, traits y tipos compartidos deben exponerse desde la crate raíz cuando formen parte del contrato público.
- La documentación principal debe guiar al usuario desde la crate raíz y no desde crates internas individuales.
