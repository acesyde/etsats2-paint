## Palabras compartidas, tiempos relativos y ajustes.

language-system = Idioma del sistema

time-just-now = Ahora mismo
time-ago-minutes = { $count ->
    [one] hace { $count } minuto
   *[other] hace { $count } minutos
 }
time-ago-hours = { $count ->
    [one] hace { $count } hora
   *[other] hace { $count } horas
 }
time-ago-days = { $count ->
    [one] hace { $count } día
   *[other] hace { $count } días
 }

time-yesterday = Ayer
time-ago-months = { $count ->
    [one] hace { $count } mes
   *[other] hace { $count } meses
 }
time-ago-years = { $count ->
    [one] hace { $count } año
   *[other] hace { $count } años
 }

## Controles

mixed = Mixto
swatch-fill-tip = Relleno (X para alternar)
swatch-stroke-tip = Trazo (X para alternar)
picker-saturation-value = Saturación y valor
picker-opacity = Opacidad del color
gradient-bar = Barra de degradado
gradient-stop = Parada { $index } en { $location } %
