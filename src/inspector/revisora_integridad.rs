use std::path::Path;
use serde_json::Value;

/// Verifica que todos los directorios esperados existan.
/// Lee la configuración desde estructura-base.json.
pub fn ejecutar(ruta_base: &Path) -> Result<Vec<String>, String> {
    let mut faltantes = Vec::new();
    
    // Leer configuración desde JSON (igual que estructuradora_base)
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
            faltantes.push(dir_str.to_string());
        }
    }
    
    Ok(faltantes)
}