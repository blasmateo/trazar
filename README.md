# Resumen del Proyecto: Trazar

## Propósito

**Trazar** es una herramienta CLI para gestión de trazabilidad académica. Permite administrar cursos, cursantes, asistencias, quizzes, asignaciones y pagos, con el objetivo de generar reportes y métricas sobre el desempeño académico.

## Arquitectura

### Estructura de Módulos

```
src/
├── main.rs              # CLI principal con subcomandos POSIX
├── inspector/           # Gestión de estructura de datos
│   ├── init.rs          # Crear estructura base
│   ├── verificar.rs     # Verificar integridad
│   ├── purgar.rs        # Eliminar datos de usuario
│   ├── validar.rs       # Validar archivos importados
│   └── consolidar.rs    # Consolidar datos a cursos
├── curso/               # Gestión de cursos
│   ├── nuevo.rs         # Crear curso
│   ├── mostrar.rs       # Ver cursos
│   ├── editar.rs        # Editar curso
│   └── remover.rs       # Eliminar cursos
├── cursante/            # Gestión de cursantes
│   ├── nuevo.rs         # Agregar cursante
│   ├── mostrar.rs       # Ver cursantes
│   ├── editar.rs        # Editar cursante
│   └── remover.rs       # Eliminar cursantes
└── archivo/             # Gestión de archivos de datos
    ├── importar.rs      # Importar archivos crudos
    ├── exportar.rs      # Exportar datos consolidados
    ├── mostrar.rs       # Listar archivos
    └── remover.rs       # Remover archivos
```

### Estructura de Datos

```
datos/
├── cursos/
│   └── <ID>-<nombre-curso>/
│       ├── curso-info0.json
│       ├── cursantes/
│       │   └── <ID>-<nombre-cursante>/
│       │       └── cursante-info0.json
│       ├── archivo/
│       │   ├── asistencias/
│       │   │   ├── <ID>-<nombre-asignatura>
│       │   │   └── clase-<NNN>.txt
│       │   ├── quizzes/
│       │   ├── asignaciones/
│       │   └── pagos/
│       ├── asignaturas/
│       │   └── <ID>-<nombre-asignatura>/
│       │       ├── asistencias.json
│       │       ├── quizzes.json
│       │       └── asignaciones.json
│       ├── eventos-academicos/
│       │   ├── asistencias.json
│       │   └── quizzes.json
│       ├── asignaciones.json
│       └── pagos/
│           ├── comprobantes/
│           │   └── cursante-<ID>-<YYYYMMDD>-<YYYYMMDD>-sec<NNN>.png
│           └── pagos.json
│
├── entrada/
└── salida/
    └── reportes/
```

## Funcionamiento Actual

### Comandos Principales

```bash
# Inspector
trazar inspector init                    # Crear estructura base
trazar inspector verificar               # Verificar integridad
trazar inspector purgar                  # Purgar datos (requiere confirmación)
trazar inspector validar [TIPO]          # Validar archivos importados
trazar inspector consolidar -c <ID> [TIPO]  # Consolidar a cursos

# Curso
trazar curso nuevo                       # Crear curso (interactivo)
trazar curso mostrar [-i <ID>]           # Ver cursos
trazar curso editar [-i <ID>]            # Editar curso
trazar curso remover [-i <ID>...]        # Eliminar cursos

# Cursante
trazar cursante [-c <ID>] nuevo          # Agregar cursante
trazar cursante [-c <ID>] mostrar [-i <ID>]  # Ver cursantes
trazar cursante [-c <ID>] editar [-i <ID>]   # Editar cursante
trazar cursante [-c <ID>] remover [-i <ID>...] # Eliminar cursantes

# Archivo
trazar archivo importar <TIPO> <ARCHIVO> # Importar archivo crudo
trazar archivo exportar <TIPO>           # Exportar datos
trazar archivo mostrar [TIPO]            # Listar archivos
trazar archivo remover <ARCHIVO>         # Remover archivo

# Completions
trazar completions <SHELL> [RUTA]        # Generar autocompletado
```

### Flujo de Importación de Datos

1. **Importar**: `trazar archivo importar asistencias archivo.txt`
    - Valida formato estricto del archivo (requiere `x - Nombre` o `s - Nombre`)
    - Detecta automáticamente curso y asignatura por cabeceras
    - Coincidencia exacta del nombre del curso (kebab-case)
    - Si hay errores de formato, pregunta si importar solo los válidos
    - Copia archivo directamente a `datos/cursos/<id-curso>/archivo/asistencias/`
    - Maneja BOM invisible en archivos UTF-8

2. **Validar**: `trazar inspector validar asistencias`
    - Valida formato semántico de archivos importados
    - Verifica cabeceras obligatorias
    - Detecta errores de formato

### Formato de Archivos de Asistencias

```
# log: asistencias
# curso: <nombre-curso>
# asignatura: <nombre-asignatura>
# clase: <numero>
# fecha_creacion: <YYYYMMDDTHHMMSS±Z>
# ====================

x - Nombre Apellido Uno
s - Nombre Apellido Dos
x - Nombre Apellido Tres
```

**Cabeceras:**
- `log`: Obligatorio, debe ser "asistencias"
- `curso`: Opcional, nombre del curso
- `asignatura`: Opcional, nombre de asignatura (si se omite va a eventos-academicos)
- `clase`: Opcional, número de clase (también se extrae del nombre del archivo con patrón `cNNN`)
- `fecha_creacion`: Opcional, timestamp ISO

**Líneas de asistencia:**
- Formato: `[x|s|X|S] - <Nombre Completo>` (estricto: requiere x/s/S/X)
- `x`/`X`: ausente
- `s`/`S`: presente

### Estructura JSON de Datos Consolidados

#### Asistencias (`asistencias.json`)
```json
{
  "dataset": "asistencias",
  "version": 0,
  "cursoId": 1,
  "asignaturaId": null,
  "clases": {
    "clase-001": {
      "cursante-001": "presente",
      "cursante-002": "ausente"
    }
  }
}
```

**Estados:** `presente`, `ausente`, `demora`, `justifica`

#### Quizzes (`quizzes.json`)
```json
{
  "dataset": "quizzes",
  "version": 0,
  "cursoId": 1,
  "asignaturaId": 1,
  "clases": {
    "clase-001": {
      "quiz-001": {
        "cuestion": "Introducción a fábulas",
        "respuestaCorrecta": "b",
        "resultados": {
          "cursante-001": 85,
          "cursante-002": 92
        }
      }
    }
  }
}
```

#### Asignaciones (`asignaciones.json`)
```json
{
  "dataset": "asignaciones",
  "version": 0,
  "cursoId": 1,
  "asignaturaId": null,
  "asignaciones": {
    "asignacion-001": {
      "titulo": "Ensayo sobre fábulas",
      "fechaInicio": "2026-06-15",
      "fechaEntrega": "2026-06-20",
      "fechaLimite": "2026-06-25",
      "calificacionMaxima": 100,
      "estadoCursantes": {
        "cursante-001": {
          "estado": "calificado",
          "calificacion": 90,
          "notas": "Buen trabajo"
        }
      }
    }
  }
}
```

**Estados:** `pendiente`, `entregado`, `calificado`, `vencido`

#### Pagos (`pagos.json`)
```json
{
  "dataset": "pagos",
  "version": 0,
  "cursoId": 1,
  "cursantes": {
    "cursante-001": {
      "pago-001": {
        "monto": 70.00,
        "moneda": "USD",
        "fechaRegistro": "2026-06-15T10:30:00-05:00",
        "periodoInicio": "20260601",
        "periodoFin": "20260731",
        "tieneComprobante": true,
        "rutaComprobante": "comprobantes/cursante-001-20260601-20260731-sec001.png",
        "notas": "Pago de junio y julio"
      }
    }
  }
}
```

## Convenciones de Diseño

### IDs y Prefijos
- **Valores numéricos** (cursoId, asignaturaId): sin ceros, secuenciales (`1`, `2`, `3`)
- **Claves de objetos**: con prefijo + 3 dígitos (`"cursante-001"`, `"clase-001"`)

### Nombres de Archivos
- **JSON**: kebab-case (`asistencias.json`, `curso-info0.json`)
- **Carpetas**: `<ID>-<nombre-kebab>` (`001-fabulas`, `001-nombre-apellido`)
- **Comprobantes**: `cursante-<ID>-<YYYYMMDD>-<YYYYMMDD>-sec<NNN>.png`

### Claves JSON
- **Convención**: camelCase (`cursoId`, `fechaInicio`, `calificacionMaxima`)

### Inmutabilidad de IDs
Los IDs de cursantes son **inmutables**. Una vez asignados, nunca cambian. Esto garantiza integridad referencial en todos los datasets.

## Proyección Futura

### Módulos Planeados

1. **Métricas** (`trazar metricas`)
   - Generar reportes de desempeño
   - Estadísticas de asistencia
   - Análisis de calificaciones
   - Reportes financieros

2. **Exportación Avanzada**
   - Exportar a PDF
   - Exportar a Excel/CSV
   - Formatos personalizados

3. **Validación Mejorada**
   - Validación cruzada entre datasets
   - Detección de inconsistencias
   - Sugerencias de corrección

4. **Integración con Fuentes Externas**
   - Importar desde Google Forms (quizzes)
   - Importar desde sistemas de pago
   - Sincronización con calendarios

### Tipos de Dataset Futuros

- **Quizzes**: Actualmente en diseño, pendiente implementación de importación
- **Asignaciones**: Actualmente en diseño, pendiente implementación de importación
- **Pagos**: Actualmente en diseño, pendiente implementación de importación

### Mejoras de UX

- Autocompletado inteligente de nombres de cursantes
- Búsqueda fuzzy en listas interactivas
- Modo batch para operaciones masivas
- Dashboard interactivo en terminal

## Dependencias Técnicas

- **Rust** (lenguaje)
- **clap** (parsing de CLI)
- **serde_json** (manejo de JSON)
- **rustyline** (editor interactivo)
- **regex** (validación de formatos)
- **clap_complete** (generación de autocompletado)

## Notas de Implementación

- Todos los comandos tienen ayuda en español (`--help` para detallada, `-h` para resumen)
- Las operaciones destructivas requieren confirmación explícita
- Los modos interactivos se activan cuando faltan argumentos requeridos
- La validación es estricta pero con mensajes de error descriptivos
- Los archivos se organizan jerárquicamente para facilitar navegación manual
