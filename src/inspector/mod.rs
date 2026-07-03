// MÓDULO INSPECTOR

mod init;
mod verificar;
mod purgar;
mod validar;
mod consolidar;

pub use init::ejecutar as init;
pub use verificar::ejecutar as verificar;
pub use purgar::ejecutar as purgar;
pub use validar::ejecutar as validar;
pub use consolidar::ejecutar as consolidar;