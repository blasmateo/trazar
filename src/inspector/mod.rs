// MÓDULO INSPECTOR

mod init;
mod verificar;
mod purgar;

pub use init::ejecutar as init;
pub use verificar::ejecutar as verificar;
pub use purgar::ejecutar as purgar;