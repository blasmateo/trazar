use std::fs;
use std::path::Path;
use serde_json::Value;

/// Crea la estructura base de directorios si no existe.
/// Lee la configuración desde estructura-base.json.
pub fn ejecutar(ruta_base: &Path) -> Result<(), String> {
    // Ruta al JSON de configuración (embebido en el binario)
    let config_json = include_str!("estructura-base.json");
    let config: Value = serde_json::from_str(config_json)
        .map_err(|e| format!("Error al leer configuración: {}", e))?;
    
    let directorios = config["directorios"]
        .as_array()
        .ok_or("Configuración inválida: falta 'directorios'")?;
    
    for dir in directorios {
        let dir_str = dir.as_str().ok_or("Directorio inválido en configuración")?;
        let ruta_completa = ruta_base.join(dir_str);
        
        if !ruta_completa.exists() {
            fs::create_dir_all(&ruta_completa)
                .map_err(|e| format!("Error al crear {}: {}", dir_str, e))?;
            println!("✓ Creado: {}", dir_str);
        }
    }
    
    Ok(())
}