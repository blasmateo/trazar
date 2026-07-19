use std::fs;
use std::path::Path;

/// Valida archivos de asistencias en datos/cursos/*/archivo/asistencias/
pub fn ejecutar(ruta_base: &Path, tipo_str: Option<&str>) -> Result<(), String> {
    if let Some(tipo) = tipo_str {
        if tipo != "asistencias" {
            return Err(format!("Validación de '{}' aún no implementada. Solo 'asistencias' está disponible.", tipo));
        }
    }
    
    let ruta_cursos = ruta_base.join("datos/cursos");
    
    if !ruta_cursos.exists() {
        return Err("No existe el directorio datos/cursos/".to_string());
    }
    
    let mut total_archivos = 0;
    let mut errores = Vec::new();
    
    // Iterar por todos los cursos
    let entradas_cursos = fs::read_dir(&ruta_cursos)
        .map_err(|e| format!("Error al leer directorio de cursos: {}", e))?;
    
    for entrada_curso in entradas_cursos {
        let entrada_curso = entrada_curso.map_err(|e| format!("Error al leer entrada: {}", e))?;
        let ruta_curso = entrada_curso.path();
        
        if !ruta_curso.is_dir() {
            continue;
        }
        
        // Buscar archivos en datos/cursos/<curso>/archivo/asistencias/
        let ruta_asistencias = ruta_curso.join("archivo/asistencias");
        
        if !ruta_asistencias.exists() {
            continue;
        }
        
        // Validar archivos en el directorio de asistencias (incluyendo subdirectorios de asignaturas)
        validar_directorio_asistencias(&ruta_asistencias, &mut total_archivos, &mut errores)?;
    }
    
    println!("Archivos validados: {}", total_archivos);
    if !errores.is_empty() {
        for err in &errores {
            eprintln!("✗ {}", err);
        }
        return Err(format!("{} archivo(s) con errores de validación", errores.len()));
    }
    
    Ok(())
}

fn validar_directorio_asistencias(ruta: &Path, total: &mut usize, errores: &mut Vec<String>) -> Result<(), String> {
    let entradas = fs::read_dir(ruta)
        .map_err(|e| format!("Error al leer directorio {}: {}", ruta.display(), e))?;
    
    for entrada in entradas {
        let entrada = entrada.map_err(|e| format!("Error al leer entrada: {}", e))?;
        let ruta_item = entrada.path();
        
        if ruta_item.is_dir() {
            // Recursivamente validar subdirectorios (asignaturas)
            validar_directorio_asistencias(&ruta_item, total, errores)?;
        } else if ruta_item.is_file() {
            *total += 1;
            if let Err(e) = validar_archivo_asistencia(&ruta_item) {
                errores.push(format!("{}: {}", ruta_item.display(), e));
            }
        }
    }
    
    Ok(())
}

fn validar_archivo_asistencia(ruta: &Path) -> Result<(), String> {
    let contenido = fs::read_to_string(ruta)
        .map_err(|e| format!("Error al leer archivo: {}", e))?;
    
    // Validación básica: verificar cabeceras requeridas
    let mut log_encontrado = false;
    for linea in contenido.lines() {
        let linea_trim = linea.trim();
        
        if linea_trim.is_empty() || linea_trim.starts_with('#') {
            if linea_trim.starts_with("# log:") {
                if let Some(valor) = linea_trim.strip_prefix("# log:") {
                    let valor_trim = valor.trim();
                    if valor_trim == "asistencias" {
                        log_encontrado = true;
                    }
                }
            }
        }
    }
    
    if !log_encontrado {
        return Err("Falta la cabecera '# log: asistencias' obligatoria".to_string());
    }
    
    Ok(())
}