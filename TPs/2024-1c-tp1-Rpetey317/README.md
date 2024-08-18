# Programación concurrente - TP1

## 1.- Características funcionales

- El programa debe procesar todos los archivos con extensión `.jsonl` en el directorio `data`
- Recibe por línea de comandos un único parámetro que corresponde a la cantidad de worker threads a utilizar
- Imprime un resultado por consola del siguiente formato:

```
{
    "padron": <número de padron del alumno>,
    "sites": {
        "site1": {
            "questions": <cantidad total de preguntas para ese sitio>,
            "words": <cantidad total de palabras para ese sitio>,
            "tags": {
                "tag1": {
                    "questions": <cantidad total de preguntas para ese tag para ese sitio>,
                    "words": <cantidad total palabras para ese tag para ese sitio>,
                },
                ...
                "tagN": {

                },
            }
            "chatty_tags": [
                "tag1", "tag2", ... // los 10 tags con mayor relación words/questions para ese sitio
            ]
        },
        ...
        "siteN" : {
            ...
        }
    },
    "tags": {
        "tag1": {
            "questions": <cantidad total de preguntas para ese tag para todos los sitios>,
            "words": <cantidad total palabras para ese tag para todos los sitios>,
        },
        ...
        "tagN": {

        },
    },
    "totals": {
        "chatty_sites": [
            "site1", "site2", ... // los 10 sitios con mayor relación words/questions
        ],
        "chatty_tags": [
            "tag1", "tag2", ... // los 10 tags con mayor relación words/questions entre todos los sitios.
        ]
    }
}
```

## 2.- Requerimientos no funcionales

- El proyecto deberá ser desarrollado en lenguaje Rust, usando las herramientas de la biblioteca estándar.
- El archivo `Cargo.toml` se debe encontrar en la raíz del repositorio, para poder ejecutar correctamente los tests automatizados
- Se deberán utilizar las herramientas de concurrencia correspondientes al modelo _forkjoin_
- No se permite utilizar crates externos, salvo los explícitamente mencionados en este enunciado, en los ejemplos de la materia, o autorizados expresamente por los profesores. Para el procesamiento de `JSON` se puede utilizar el crate `serde_json`.
- El código fuente debe compilarse en la última versión stable del compilador y no se permite utilizar bloques unsafe.
- El código deberá funcionar en ambiente Unix / Linux.
- El programa deberá ejecutarse en la línea de comandos.
- La compilación no debe arrojar warnings del compilador, ni del linter clippy.
- Las funciones y los tipos de datos (struct) deben estar documentadas siguiendo el estándar de cargo doc.
- El código debe formatearse utilizando `cargo fmt`.
- Cada tipo de dato implementado debe ser colocado en una unidad de compilación (archivo fuente) independiente.

## 3.- Requerimientos de entrega

El trabajo se desarrolla de forma individual, hasta el **17 de abril del 2024, 19hs Arg (GMT -4)**
