# TRAZAR

**Trazabilidad Académica y Reporte**

Aplicación de terminal para gestión de estudiantes, cursos, asistencias y pagos.

## Arquitectura

- **Rust:** CLI principal y gestión de archivos
- **Python:** Generación de documentos (PDF/Word) y análisis estadístico

## Uso

```bash
# Compilar Rust
cargo build --release

# Ejecutar
./target/release/trazar <comando>
```

# Comandos
trazar inspector -e    # Crear estructura base
trazar inspector -r    # Verificar integridad
trazar inspector -l    # Eliminar datos de usuario