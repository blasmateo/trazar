use std::path::Path;

/// Consolida archivos validados a datos/cursos/
pub fn ejecutar(ruta_base: &Path, curso_arg: &str, tipo_str: Option<&str>) -> Result<(), String> {
    let ruta_cursos = ruta_base.join("datos/cursos");
    
    if !ruta_cursos.exists() {
        return Err("No existe el directorio datos/cursos/".to_string());
    }
    
    // TODO: Implementar consolidación
    if let Some(tipo) = tipo_str {
        println!("Consolidación de '{}' al curso '{}' aún no implementada.", tipo, curso_arg);
    } else {
        println!("Consolidación de todos los tipos al curso '{}' aún no implementada.", curso_arg);
    }
    
    Ok(())
}