## Títulos de los paneles y estados vacíos

panel-properties = Propiedades
panel-layers = Capas
panel-colors = Colores
panel-stroke = Trazo
panel-transform = Transformar
panel-assets = Recursos
panel-vehicle = Vehículos
empty-properties = Nada seleccionado
empty-properties-hint = Seleccione un objeto en el lienzo para editar sus propiedades.
empty-layers = Aún no hay capas
empty-layers-hint = Las formas, los textos y las imágenes que añada aparecerán aquí.
empty-colors = Ningún color seleccionado
empty-colors-hint = Elija un relleno o un trazo para editar su color.
empty-stroke = Sin trazo
empty-stroke-hint = Seleccione un objeto para editar su contorno.
empty-transform = Nada que transformar
empty-transform-hint = Seleccione un objeto para editar su posición, tamaño y rotación.
empty-assets = No hay recursos
empty-assets-hint = Los logotipos y las imágenes importados aparecerán aquí.
empty-vehicle = Ningún vehículo
empty-vehicle-hint = Las plantillas de vehículos estarán disponibles en una próxima actualización.

## Vista previa 3D

preview-hide = Ocultar vista previa 3D
preview-soon = Vista previa 3D próximamente
preview-soon-hint = Su librea se mostrará aquí sobre el modelo del vehículo cuando los modelos de vehículos estén disponibles.

## Paneles y capas

panels-all-closed = Todos los paneles están cerrados
panels-all-closed-hint = Vuelva a abrir los paneles desde el menú Ver o restablezca el espacio de trabajo.
panel-expand = Expandir
panel-collapse = Contraer
panel-close = Cerrar panel
layers-empty-hint = Las formas que dibuje y las capas que añada aparecerán aquí.
layers-collapse = Contraer { $name }
layers-expand = Expandir { $name }
layers-hide = Ocultar { $name }
layers-show = Mostrar { $name }
layers-lock = Bloquear { $name }
layers-unlock = Desbloquear { $name }
layers-name = Nombre de la capa

## Recursos

assets-uses = { $count ->
    [one] { $count } uso
   *[other] { $count } usos
 }
assets-used-by = { $count ->
    [one] Usado por { $count } objeto
   *[other] Usado por { $count } objetos
 }
assets-vector = Vectorial
assets-item = Recurso { $name }
assets-place = Colocar recurso
assets-name = Nombre del recurso

## Panel Trazo

stroke-enabled = Trazo activado
stroke-width = Grosor del trazo
stroke-center = Centro
stroke-center-name = Trazo centrado
stroke-inside = Interior
stroke-inside-name = Trazo interior
stroke-outside = Exterior
stroke-outside-name = Trazo exterior
undo-change-stroke-width = Cambiar grosor del trazo
undo-change-stroke-alignment = Cambiar alineación del trazo
undo-add-stroke = Añadir trazo
undo-remove-stroke = Quitar trazo

## Panel Transformar

field-width = Anchura
field-height = Altura
field-w = An
field-h = Al
field-r = R
field-s = E
transform-x = Posición X
transform-y = Posición Y
transform-rotation = Rotación
transform-scale = Escala
transform-unlock-proportions = Desbloquear proporciones
transform-lock-proportions = Bloquear proporciones
transform-align-to = Alinear con

## Panel Propiedades

undo-change-opacity = Cambiar opacidad
undo-change-corner-radius = Cambiar radio de esquina
undo-change-sides = Cambiar lados
undo-change-inner-radius = Cambiar radio interior
undo-change-line-width = Cambiar grosor de línea
undo-change-star = Cambiar estrella
props-sides = Lados
props-star = Estrella
props-inner = Interior
props-inner-radius = Radio interior
props-surface-summary = { $size } × { $size } px · Seleccione un objeto para editarlo
props-objects = { $count } objetos
props-multiple = Selección múltiple
props-source-svg = SVG · Vectorial
props-source-image = Imagen · { $width } × { $height } px
props-source = Origen: { $name } · { $source }
props-opacity = Opacidad
props-opacity-slider = Control deslizante de opacidad
props-radius = Radio
props-corner-radius = Radio de esquina
props-line-width = Grosor de línea
props-fill = Relleno
props-fill-color = Color de relleno
props-stroke-color = Color de trazo

## Guiones, extremos y uniones

line-preset-dashed = Discontinua 20/10
line-preset-dotted = Punteada 0/12
line-preset-long-dash = Guion largo 60/20
undo-change-stroke-caps = Cambiar extremos del trazo
undo-change-stroke-joins = Cambiar uniones del trazo
undo-change-stroke-dashes = Cambiar guiones del trazo
undo-change-line-caps = Cambiar extremos de línea
undo-change-line-joins = Cambiar uniones de línea
undo-change-line-dashes = Cambiar guiones de línea
line-dashed = Discontinua
line-presets = Ajustes preestablecidos
line-dash-presets = Guiones preestablecidos
line-dash = Guion
line-dash-length = Longitud del guion
line-gap = Hueco
line-gap-length = Longitud del hueco
line-cap = Extremo
line-butt = Plano
line-butt-cap = Extremo plano
line-round = Redondeado
line-round-cap = Extremo redondeado
line-square = Cuadrado
line-square-cap = Extremo cuadrado
line-join = Unión
line-miter = En ángulo
line-miter-join = Unión en ángulo
line-round-join = Unión redondeada
line-bevel = Biselada
line-bevel-join = Unión biselada
line-limit = Límite
line-miter-limit = Límite de ángulo

## Panel Colores

colors-no-stroke = Sin trazo
colors-solid = Sólido
colors-solid-paint = Color sólido
colors-linear = Lineal
colors-linear-gradient = Degradado lineal
colors-radial = Radial
colors-radial-gradient = Degradado radial
colors-location = Posición
colors-stop-location = Posición de la parada
colors-angle = Ángulo
colors-gradient-angle = Ángulo del degradado
colors-reverse = Invertir degradado
colors-red = Rojo
colors-green = Verde
colors-blue = Azul
colors-hue = Tono
colors-saturation = Saturación
colors-value = Valor
colors-lightness = Luminosidad
colors-alpha = Alfa
colors-hex = Hex
colors-hex-color = Color hexadecimal
colors-hex-invalid = Color hexadecimal no válido. Use #RGB, #RRGGBB o #RRGGBBAA.
colors-recent = Recientes
colors-recent-item = Color reciente { $color }
colors-palette = Paleta
colors-palette-item = Color de la paleta { $color }
colors-add-to-palette = Añadir a la paleta
colors-differ = Los colores difieren en la selección.
colors-palette-empty = Guarde con + los colores que reutiliza.
colors-remove-from-palette = Quitar de la paleta
colors-aspect = Proporción
colors-aspect-ratio = Proporción del degradado

## Panel Carácter

weight-thin = Fina
weight-extra-light = Extraligera
weight-light = Ligera
weight-regular = Normal
weight-medium = Media
weight-semi-bold = Seminegrita
weight-bold = Negrita
weight-extra-bold = Extranegrita
weight-black = Black
char-title = Carácter
char-font-weight = Grosor de la fuente
char-italic = Cursiva
char-size = Tamaño
char-font-size = Tamaño de la fuente
char-align-left = Alinear a la izquierda
char-align-center = Centrar
char-align-right = Alinear a la derecha
char-tracking = Espaciado
char-letter-spacing = Espaciado entre letras
char-line = Línea
char-line-height = Interlineado
char-family-missing-name = Familia de fuentes: { $family } (Fuente no encontrada: { $family })
char-family-name = Familia de fuentes: { $family }
char-font-not-found = Fuente no encontrada: { $family }
char-font-family = Familia de fuentes
char-search-fonts = Buscar fuentes
char-no-fonts = No hay fuentes que coincidan
undo-change-font-weight = Cambiar grosor de la fuente
undo-change-italic = Cambiar cursiva
undo-change-text-size = Cambiar tamaño del texto
undo-change-alignment = Cambiar alineación
undo-change-letter-spacing = Cambiar espaciado entre letras
undo-change-line-height = Cambiar interlineado
undo-change-font = Cambiar fuente

## Vehicle panel

vehicle-panel-package = Paquete { $version } · versiones del juego { $games }
vehicle-panel-package-missing = Paquete { $version } (no instalado)
vehicle-panel-update = La versión { $version } está disponible
vehicle-panel-texture = Textura { $name }
vehicle-panel-layout-changed = Disposición cambiada
vehicle-panel-dismiss = Descartar
vehicle-panel-dismiss-named = Descartar el cambio de disposición de { $name }
vehicle-panel-removed = No está en esta versión
vehicle-panel-opacity = Plantilla
vehicle-panel-opacity-name = Opacidad de la plantilla

## Fleet

vehicles-fleet-game = Flota { $game }
vehicle-panel-update-named = Actualizar la plantilla de { $name }
vehicles-variants = Variantes…
vehicles-variants-named = Variantes de { $name }
vehicles-remove-from-project = Quitar del proyecto
vehicles-remove-named = Quitar { $name } del proyecto

## Vehicles sidebar

vehicles-sidebar-show = Mostrar vehículos
vehicles-sidebar-hide = Ocultar vehículos
