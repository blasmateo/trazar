use std::path::Path;

/// Remueve un archivo de datos/archivo/
pub fn ejecutar(_ruta_base: &Path, archivo: &str) -> Result<(), String> {
    let _archivo_path = Path::new(archivo);
    
    // TODO: Implementar lógica de remoción
    Err("Remoción aún no implementada".to_string())
}