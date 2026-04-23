use crate::expr::{Expr, TableRef};
use crate::join::Join;
use crate::order::OrderBy;
use crate::pagination::Pagination;
use crate::predicate::Predicate;
use mssql_orm_core::Entity;

#[derive(Debug, Clone, PartialEq)]
pub struct SelectQuery {
    pub from: TableRef,
    pub joins: Vec<Join>,
    pub projection: Vec<Expr>,
    pub predicate: Option<Predicate>,
    pub order_by: Vec<OrderBy>,
    pub pagination: Option<Pagination>,
}

impl SelectQuery {
    pub fn from_entity<E: Entity>() -> Self {
        Self {
            from: TableRef::for_entity::<E>(),
            joins: Vec::new(),
            projection: Vec::new(),
            predicate: None,
            order_by: Vec::new(),
            pagination: None,
        }
    }

    pub fn select(mut self, projection: Vec<Expr>) -> Self {
        self.projection = projection;
        self
    }

    pub fn filter(mut self, predicate: Predicate) -> Self {
        self.predicate = Some(match self.predicate.take() {
            Some(existing) => Predicate::and(vec![existing, predicate]),
            None => predicate,
        });
        self
    }

    pub fn join(mut self, join: Join) -> Self {
        self.joins.push(join);
        self
    }

    pub fn inner_join<E: Entity>(self, on: Predicate) -> Self {
        self.join(Join::inner_entity::<E>(on))
    }

    pub fn left_join<E: Entity>(self, on: Predicate) -> Self {
        self.join(Join::left_entity::<E>(on))
    }

    pub fn order_by(mut self, order: OrderBy) -> Self {
        self.order_by.push(order);
        self
    }

    pub fn paginate(mut self, pagination: Pagination) -> Self {
        self.pagination = Some(pagination);
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountQuery {
    pub from: TableRef,
    pub predicate: Option<Predicate>,
}

impl CountQuery {
    pub fn from_entity<E: Entity>() -> Self {
        Self {
            from: TableRef::for_entity::<E>(),
            predicate: None,
        }
    }

    pub fn filter(mut self, predicate: Predicate) -> Self {
        self.predicate = Some(match self.predicate.take() {
            Some(existing) => Predicate::and(vec![existing, predicate]),
            None => predicate,
        });
        self
    }
}
