use std::fs;
use std::path::Path;

/// Lista archivos en datos/cursos/*/archivo/
pub fn ejecutar(ruta_base: &Path, tipo_str: Option<&str>) -> Result<(), String> {
    if let Some(tipo) = tipo_str {
        if tipo != "asistencias" {
            return Err(format!("Mostrar de '{}' aún no implementado. Solo 'asistencias' está disponible.", tipo));
        }
    }
    
    let ruta_cursos = ruta_base.join("datos/cursos");
    
    if !ruta_cursos.exists() {
        return Err("No existe el directorio datos/cursos/. Ejecute 'trazar inspector init' primero.".to_string());
    }
    
    println!("╔══════════════════════════════════════════════════╗");
    println!("║            ARCHIVOS IMPORTADOS                   ║");
    println!("╚══════════════════════════════════════════════════╝\n");
    
    // Iterar por todos los cursos
    let entradas_cursos = fs::read_dir(&ruta_cursos)
        .map_err(|e| format!("Error al leer directorio de cursos: {}", e))?;
    
    let mut total_archivos = 0;
    
    for entrada_curso in entradas_cursos {
        let entrada_curso = entrada_curso.map_err(|e| format!("Error al leer entrada: {}", e))?;
        let ruta_curso = entrada_curso.path();
        
        if !ruta_curso.is_dir() {
            continue;
        }
        
        if let Some(nombre) = ruta_curso.file_name() {
            println!("Curso: {}", nombre.to_string_lossy());
        }
        
        // Mostrar archivos en datos/cursos/<curso>/archivo/asistencias/
        let ruta_archivo = ruta_curso.join("archivo/asistencias");
        
        if ruta_archivo.exists() {
            listar_archivos_asistencias(&ruta_archivo, &mut total_archivos)?;
        }
        
        println!();
    }
    
    println!("Total de archivos: {}", total_archivos);
    Ok(())
}

fn listar_archivos_asistencias(ruta: &Path, total: &mut usize) -> Result<(), String> {
    let entradas = fs::read_dir(ruta)
        .map_err(|e| format!("Error al leer directorio {}: {}", ruta.display(), e))?;
    
    for entrada in entradas {
        let entrada = entrada.map_err(|e| format!("Error al leer entrada: {}", e))?;
        let ruta_item = entrada.path();
        
        if ruta_item.is_dir() {
            // Subdirectorios son asignaturas
            if let Some(nombre) = ruta_item.file_name() {
                println!("  ├─ Asignatura: {}", nombre.to_string_lossy());
            }
            listar_archivos_en_asignatura(&ruta_item, total)?;
        } else if ruta_item.is_file() {
            *total += 1;
            if let Some(nombre) = ruta_item.file_name() {
                println!("  └─ {}", nombre.to_string_lossy());
            }
        }
    }
    
    Ok(())
}

fn listar_archivos_en_asignatura(ruta: &Path, total: &mut usize) -> Result<(), String> {
    let entradas = fs::read_dir(ruta)
        .map_err(|e| format!("Error al leer directorio {}: {}", ruta.display(), e))?;
    
    for entrada in entradas {
        let entrada = entrada.map_err(|e| format!("Error al leer entrada: {}", e))?;
        let ruta_item = entrada.path();
        
        if ruta_item.is_file() {
            *total += 1;
            if let Some(nombre) = ruta_item.file_name() {
                println!("  │  └─ {}", nombre.to_string_lossy());
            }
        }
    }
    
    Ok(())
}