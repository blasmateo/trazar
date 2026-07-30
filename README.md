# Resumen del Proyecto: Trazar

## Propósito

**Trazar** es una herramienta CLI para gestión de trazabilidad académica. Permite administrar cursos, cursantes, asistencias, quizzes, asignaciones y pagos, con el objetivo de generar reportes y métricas sobre el desempeño académico.

## Arquitectura

### Estructura de Módulos

```
src/
├── main.rs              # CLI principal con subcomandos POSIX
├── archivo/             # Gestión de archivos de datos e inspección de estructura
│   ├── init.rs          # Crear estructura base
│   ├── purgar.rs        # Eliminar datos de usuario
│   ├── verificar.rs     # Verificar integridad de directorios
│   ├── validar.rs       # Validar archivos importados
│   ├── importar.rs      # Importar archivos crudos
│   ├── exportar.rs      # Exportar datos a .docx (vía herramienta externa Python)
│   ├── mostrar.rs       # Listar archivos
│   └── remover.rs       # Remover archivos
├── curso/               # Gestión de cursos
│   ├── nuevo.rs         # Crear curso
│   ├── mostrar.rs       # Ver cursos
│   ├── editar.rs        # Editar curso
│   ├── preguntas.rs    # Funciones interactivas (texto, número, costo, fechas, curso, asignatura)
│   └── remover.rs       # Eliminar cursos
├── cursante/            # Gestión de cursantes
│   ├── nuevo.rs         # Agregar cursante
│   ├── mostrar.rs       # Ver cursantes
│   ├── editar.rs        # Editar cursante
│   └── remover.rs       # Eliminar cursantes
└── metricas/            # Generación de métricas
    └── calcular.rs      # Calcular estadísticas
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
│       │   │   ├── <ID>-<nombre-asignatura>/
│       │   │   │   └── clase-<NNN>.txt
│       │   │   └── clase-<NNN>.txt (sin asignatura)
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
│       ├── pagos/
│       │   ├── comprobantes/
│       │   │   └── cursante-<ID>-<YYYYMMDD>-<YYYYMMDD>-sec<NNN>.png
│       │   └── pagos.json
│       ├── metricas/
│       │   ├── asistencias-resumen.json
│       │   └── asistencias-tabla.json
│
├── entrada/
└── salida/
    └── reportes/
```

## Funcionamiento Actual

### Comandos Principales

```bash
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

# Archivo (gestión de datos e inspección de estructura)
trazar archivo init                                    # Crear estructura base
trazar archivo purgar                                  # Purgar datos (requiere confirmación: 'Si')
trazar archivo -i verificar                            # Verificar integridad de directorios
trazar archivo -i validar [-t <TIPO>]                  # Validar archivos importados
trazar archivo importar -t <TIPO> -r <RUTA>... [-s]    # Importar (modo interactivo si hay errores)
trazar archivo exportar -t <TIPO> -m [lista|tabla] -r <RUTA>  # Exportar datos a .docx
trazar archivo mostrar [TIPO]                          # Listar archivos
trazar archivo remover [ARCHIVO...]                    # Remover archivo (modo interactivo si no se especifica)

# Métricas
trazar metricas mostrar -t <TIPO> [-m <MODO>]         # Mostrar métricas guardadas (lee JSON)
trazar metricas calcular -t <TIPO> [-c <CURSANTE>] [-m <MODO>] [-a]  # Calcular estadísticas
  TIPOS VÁLIDOS (-t): asistencias, quizzes, asignaciones, pagos (completado automático)
  OPCIONES (calcular y mostrar):
    -c, --cursante <CURSANTE>  Filtrar por cursante (solo muestra, no afecta JSON)
    -m, --modo <MODO>          lista (resumen) o tabla (detallado)
    -a, --actualizar           Guardar resultados en JSON (no afectado por filtro)

# Completions
trazar completions <SHELL> [RUTA]        # Generar autocompletado
```

### Flujo de Importación de Datos

1. **Importar**: `trazar archivo importar -t asistencias -r archivo.txt [archivo2.txt ...]`
    - Valida formato estricto del archivo (requiere `x/s - Nombre`)
    - Clasifica errores en **formato** (contenido inválido) vs **metadata** (archivo no encontrado)
    - Detecta automáticamente curso y asignatura por cabeceras
    - Coincidencia exacta del nombre del curso (kebab-case); asignaturas con coincidencia parcial
    - Acepta archivos individuales, directorios, y globs expandidos por el shell
    - Si hay errores de formato, pregunta si importar solo los válidos (o usa `-s` para auto-afirmar)
    - Copia archivo a `datos/cursos/<id-curso>/archivo/asistencias/[<id-asignatura>-<nombre>/]clase-<NNN>.txt`
    - Maneja BOM invisible en archivos UTF-8
    - Asigna automáticamente IDs a nuevas asignaturas

2. **Validar**: `trazar archivo -i validar [-t asistencias]`
    - Valida formato semántico de archivos importados en todos los cursos
    - Verifica cabecera obligatoria `# log: asistencias`
    - Recorre recursivamente subdirectorios de asignaturas

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

## Convenciones de Diseño

### IDs y Prefijos
- **Valores numéricos** (cursoId, asignaturaId): sin ceros, secuenciales (`1`, `2`, `3`)
- **Claves de objetos**: con prefijo + 3 dígitos (`"cursante-001"`, `"clase-001"`)

### Nombres de Archivos
- **JSON**: kebab-case (`asistencias.json`, `curso-info0.json`)
- **Carpetas**: `<ID>-<nombre-kebab>` (`001-categoria-ejemplo`, `001-nombre-apellido`)
- **Comprobantes**: `cursante-<ID>-<YYYYMMDD>-<YYYYMMDD>-sec<NNN>.png`

### Claves JSON
- **Convención**: camelCase (`cursoId`, `fechaInicio`, `calificacionMaxima`)

### Inmutabilidad de IDs
Los IDs de cursantes son **inmutables**. Una vez asignados, nunca cambian. Esto garantiza integridad referencial en todos los datasets.

## Proyección Futura

### Módulos Planeados

1. **Exportación Avanzada**
   - Exportar a PDF
   - Exportar a Excel/CSV
   - Formatos personalizados

2. **Validación Mejorada**
   - Validación cruzada entre datasets
   - Detección de inconsistencias
   - Sugerencias de corrección

3. **Integración con Fuentes Externas**
   - Importar desde Google Forms (quizzes)
   - Importar desde sistemas de pago
   - Sincronización con calendarios

## Dependencias Técnicas

- **Rust** (lenguaje)
- **clap** (parsing de CLI)
- **clap_complete** (generación de autocompletado)
- **serde_json** (manejo de JSON)
- **chrono** (fechas y tiempos)
- **unicode-normalization** (normalización Unicode para nombres)
- **rustyline** (editor interactivo en terminal)
- **regex** (validación de formatos)

### Herramientas externas (Python)

- **python-docx** (generación de documentos .docx)
- **PyInstaller** (empaquetado de scripts Python en binarios autocontenidos)

## Integración con Herramientas Externas (Python)

Trazar extiende capacidades (exportación a .docx, futuros gráficos/PDF) mediante
**herramientas externas escritas en Python**, cada una en su propia subcarpeta de
`scripts/`. La integración sigue un **contrato IPC** UNIX: el binario Rust
(`trazar`) invoca al script como subproceso, comunicándose por canales estándar.

### Convención de rutas

```
<dir-ejecutable>/trazar                 # binario Rust
<dir-ejecutable>/_scripts/
    └── exportar-docx                   # binario PyInstaller (autocontenido)
```

- **Distribución**: el binario PyInstaller (en `_scripts/`) empaqueta el
  intérprete Python y sus dependencias. **No requiere Python instalado** en la
  máquina del usuario.
- **Desarrollo** (fallback): si no existe `_scripts/<nombre>`, Trazar busca
  `scripts/<nombre>/<nombre>.py` y lo ejecuta con `.venv/bin/python`.

### Contrato de comunicación

| Canal | Dirección | Contenido | Formato |
|---|---|---|---|
| stdin | Rust → Python | Datos de entrada | JSON (envelope estructurado) |
| stdout | Python → Rust | Resultado/metadata | JSON estructurado |
| stderr | Python → Rust | Logs humanos | Texto con prefijos `[INFO]`/`[WARN]`/`[ERR]` |
| Archivos | Python → FS | Artefactos binarios (.docx) | Directos al disco |
| Código de salida | Python → Rust | Semántica del resultado | `0` ok, `1` contrato, `2` entrada, `3` recursos, `4` dependencia |

Envelope de entrada (stdin):

```json
{
  "contractVersion": "1.0",
  "operation": "exportar-docx",
  "payload": { "datos": { ...métricas... } },
  "output": { "ruta": "/abs/path.docx", "modo": "lista|tabla" }
}
```

Respuesta (stdout):

```json
{ "status": "ok", "artefactos": [{ "ruta": "...", "tipo": "docx", "tamanoBytes": 12345 }] }
```

### Empaquetado

Cada herramienta externa incluye un `build.sh` que genera el binario PyInstaller:

```bash
scripts/exportar-docx/build.sh [destino]   # por defecto target/release
```

## Notas de Implementación

- Todos los comandos tienen ayuda en español (`--help` para detallada, `-h` para resumen)
- Las operaciones destructivas requieren confirmación explícita
- Los modos interactivos se activan cuando faltan argumentos requeridos
- La validación es estricta pero con mensajes de error descriptivos
- Los archivos se organizan jerárquicamente para facilitar navegación manual