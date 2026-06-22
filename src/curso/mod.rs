// MÓDULO CURSO

mod preguntas;
mod agregar;
mod ver;
mod borrar;
mod editar;

pub use agregar::ejecutar as agregar;
pub use ver::ejecutar as ver;
pub use borrar::ejecutar as borrar;
pub use editar::ejecutar as editar;