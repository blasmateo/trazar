use std::fs;
use std::path::Path;
use rustyline::DefaultEditor;

/// Elimina toda la carpeta de datos del usuario, con confirmación.
pub fn ejecutar(ruta_base: &Path) -> Result<(), String> {
    let ruta_datos = ruta_base.join("datos");
    
    let mut rl = DefaultEditor::new()
        .map_err(|e| format!("Error al inicializar editor: {}", e))?;
    
    match rl.readline("¿Confirma purgar todos los datos? (Si/N): ") {
        Ok(confirmacion) => {
            if confirmacion.trim() != "Si" {
                println!("Operación cancelada. No se purgó nada.");
                return Ok(());
            }
        }
        Err(_) => {
            println!("\nOperación cancelada. No se purgó nada.");
            return Ok(());
        }
    }
    
    if ruta_datos.exists() {
        fs::remove_dir_all(&ruta_datos)
            .map_err(|e| format!("Error al eliminar datos/: {}", e))?;
        println!("✓ Datos eliminados");
    } else {
        println!("ℹ No existe carpeta datos/ para purgar");
    }
    
    Ok(())
}