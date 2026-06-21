use std::fs;
use std::path::Path;

/// Elimina toda la carpeta de datos del usuario.
/// NO crea nada nuevo, solo elimina.
pub fn ejecutar(ruta_base: &Path) -> Result<(), String> {
    let ruta_datos = ruta_base.join("datos");
    
    if ruta_datos.exists() {
        fs::remove_dir_all(&ruta_datos)
            .map_err(|e| format!("Error al eliminar datos/: {}", e))?;
        println!("✓ Eliminada carpeta datos/");
    } else {
        println!("ℹ No existe carpeta datos/ para eliminar");
    }
    
    Ok(())
}