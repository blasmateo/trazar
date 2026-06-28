// MÓDULO CURSANTE

mod nuevo;
mod mostrar;
mod editar;
mod remover;

pub use nuevo::ejecutar as nuevo;
pub use mostrar::ejecutar as mostrar;
pub use editar::ejecutar as editar;
pub use remover::ejecutar as remover;