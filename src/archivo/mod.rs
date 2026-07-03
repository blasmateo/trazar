// MÓDULO ARCHIVO
// Gestión de archivos de datos (importar, exportar, mostrar, remover)

mod importar;
mod exportar;
mod mostrar;
mod remover;

pub use importar::ejecutar as importar;
pub use exportar::ejecutar as exportar;
pub use mostrar::ejecutar as mostrar;
pub use remover::ejecutar as remover;

/// Tipos de dataset soportados
pub enum TipoDataset {
    Asistencias,
    Quizzes,
    Asignaciones,
    Pagos,
}

impl TipoDataset {
    /// Convierte un string a TipoDataset
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "asistencias" => Ok(TipoDataset::Asistencias),
            "quizzes" => Ok(TipoDataset::Quizzes),
            "asignaciones" => Ok(TipoDataset::Asignaciones),
            "pagos" => Ok(TipoDataset::Pagos),
            _ => Err(format!(
                "Tipo de dataset no válido: '{}'. Tipos válidos: asistencias, quizzes, asignaciones, pagos",
                s
            )),
        }
    }
    
    /// Retorna el nombre del subdirectorio en datos/archivo/
    pub fn nombre_directorio(&self) -> &str {
        match self {
            TipoDataset::Asistencias => "asistencias",
            TipoDataset::Quizzes => "quizzes",
            TipoDataset::Asignaciones => "asignaciones",
            TipoDataset::Pagos => "pagos",
        }
    }
    
    /// Retorna el nombre del dataset para el JSON
    #[allow(dead_code)]
	pub fn nombre_dataset(&self) -> &str {
        self.nombre_directorio()
    }
}