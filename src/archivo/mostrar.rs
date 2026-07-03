use std::path::Path;

/// Lista archivos en datos/archivo/
pub fn ejecutar(ruta_base: &Path, tipo_str: Option<&str>) -> Result<(), String> {
    let ruta_archivo = ruta_base.join("datos/archivo");
    
    if !ruta_archivo.exists() {
        return Err("No existe el directorio datos/archivo/. Ejecute 'trazar inspector init' primero.".to_string());
    }
    
    if let Some(tipo) = tipo_str {
        let _tipo = super::TipoDataset::from_str(tipo)?;
        println!("Listado de archivos de '{}' aún no implementado.", tipo);
    } else {
        println!("Listado de todos los archivos aún no implementado.");
    }
    
    Ok(())
}