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

Requerimientos adicionales que fueron comentados por la cátedra como motivos de reentrega:

- Las corridas con diferentes cantidad de threads presentan diferencias entre sus resultados, denotando errores de implementación en su algoritmo (o peor race conditions que no deberían suceder con este modelo)
- Las corridas con más de un thread no presentan siquiera un 10% de mejora de performance en ejecución respecto de la de un solo thread. Esto puede ser por no respetar el valor del parámetro de threads en el mejor caso. En el peor caso es por algun bloqueo (que tampoco deberia suceder en este modelo)
- Los resultados reportados matchean en menos de un 50% con los esperados. Revisar los calculos y algoritmo nuevamente.
- La app no respeta el formato de salida. Solamente debe sacar un JSON con el esquema especificado en el enunciado por STDOUT
- La app no corre con cargo run <numthreads> desde el root de su repo
- La app tira panic al correrla o se cuelga en su ejecución por VARIOS minutos
- 
## 3.- Requerimientos de entrega

El trabajo se desarrolla de forma individual, hasta el **17 de abril del 2024, 19hs Arg (GMT -4)**

## 4.- Comentarios post-entrega

La nota final de este TP fue un 8. No me dieron mucho feedback, así que solo puedo especular sobre qué exactamente haría falta corregir. Lo único que se me ocurre que podría bajar nota es que faltan varios tests de algunas estructuras (honestamente me dio flojera y estaba con otras cosas), y que además de paralelizar por _archivo_, también podría haber paralelizado por _líneas de un mismo archivo_ (i.e. en vez de tener 4 threads uno en cada archivo, tener 4 threads leyendo partes distintas del mismo archivo, o incluso 2 threads en un archivo y 2 threads en otro, o a efectos prácticos lo que decida rayon asignarle a cada thread), ya que como está ahora, es más eficiente en procesar muchos archivos ligeros pero puede ir lento con pocos archivos pesados.
