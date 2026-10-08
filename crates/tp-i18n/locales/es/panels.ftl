## Títulos de los paneles y estados vacíos

panel-properties = Propiedades
panel-layers = Capas
panel-colors = Colores
panel-styles = Estilos
panel-symbols = Símbolos
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
empty-styles = Aún no hay estilos
empty-styles-hint = Guarde apariencias para reutilizarlas en cada textura.
empty-symbols = Aún no hay símbolos
empty-symbols-hint = Convierta un logotipo o una rotulación en símbolo para colocarlo en cada textura.
empty-stroke = Sin trazo
empty-stroke-hint = Seleccione un objeto para editar su contorno.
empty-transform = Nada que transformar
empty-transform-hint = Seleccione un objeto para editar su posición, tamaño y rotación.
empty-assets = No hay recursos
empty-assets-hint = Los logotipos y las imágenes importados aparecerán aquí.
empty-vehicle = Ningún vehículo
empty-vehicle-hint = Las plantillas de vehículos estarán disponibles en una próxima actualización.

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
colors-add-to-palette = Añadir a la paleta
colors-differ = Los colores difieren en la selección.
colors-palette-empty = Guarde con + los colores que reutiliza.
colors-swatch-prefix = Color
colors-edit-swatch = Editar muestra…
colors-delete-swatch = Eliminar muestra
colors-linked-to = Vinculado a { $name }
colors-swatch-name = Nombre
colors-swatch-name-empty = Escriba un nombre para la muestra.
colors-swatch-hex = Color hexadecimal de la muestra
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

vehicle-panel-update-named = Actualizar la plantilla de { $name }
vehicles-textures = Texturas…
vehicles-main-textures = Texturas principales
vehicles-accessories = Accesorios
vehicles-remove-from-project = Quitar del proyecto

## Vehicles sidebar


## Sidebar

sidebar-project = Proyecto
sidebar-show = Mostrar la barra lateral
sidebar-hide = Ocultar la barra lateral
sidebar-vehicles-game = Vehículos · { $game }
project-name = Nombre
project-version = Versión
project-game-versions = Versiones del juego
project-game-versions-supported = Compatibles con todos los vehículos: { $versions }
project-no-common-version = Ninguna versión del juego es compatible con todos los vehículos
vehicle-actions-named = Acciones de { $name }
vehicle-panel-needs-check = Hay que revisar una textura tras una actualización
texture-layout-changed = La disposición de esta textura cambió en la versión { $version }: revise su diseño.
texture-not-in-version = Esta textura no existe en la versión { $version } del vehículo: no tiene plantilla.

## Styles panel

styles-graphic = Estilos gráficos
styles-text = Estilos de texto
styles-graphic-empty = Guarde un relleno, un trazo y una opacidad para reutilizarlos en cada textura: seleccione una forma y pulse +.
styles-text-empty = Guarde ajustes de rotulación para reutilizarlos en cada textura: seleccione un texto y pulse +.
styles-new-graphic = Nuevo estilo gráfico a partir de la selección
styles-new-text = Nuevo estilo de texto a partir de la selección
styles-new = Nuevo estilo
styles-need-shape = Seleccione una forma o un texto.
styles-need-text = Seleccione un texto.
styles-graphic-item = Estilo gráfico { $name }
styles-text-item = Estilo de texto { $name }
styles-graphic-prefix = Estilo
styles-text-prefix = Estilo de texto
styles-apply = Aplicar estilo
styles-rename = Cambiar nombre
styles-redefine = Redefinir a partir de la selección
styles-select-users = Seleccionar los usos en esta textura
styles-delete = Eliminar estilo
styles-name = Nombre del estilo

## Symbols panel

symbols-prefix = Símbolo
symbols-copy-suffix = copia
symbols-empty = Dibuje una vez un logotipo o una rotulación, selecciónelo y elija Convertir en símbolo: colóquelo en cada textura y edítelo una sola vez para todas.
symbols-instances = { $count ->
    [one] { $count } instancia
   *[other] { $count } instancias
 }
symbols-item = Símbolo { $name }
symbols-place = Colocar { $name }
symbols-edit = Editar { $name }
symbols-rename = Cambiar nombre
symbols-duplicate = Duplicar
symbols-delete = Eliminar símbolo
symbols-delete-confirm = ¿Eliminar { $name }? { $count ->
    [one] Su instancia se convierte en un grupo idéntico.
   *[other] Sus { $count } instancias se convierten en grupos idénticos.
 }
symbols-name = Nombre del símbolo
symbol-bar-editing = Editando el símbolo { $name }
symbol-bar-done = Hecho
props-instance-of = Instancia de { $name }
instance-look-in-symbol = Una instancia muestra su símbolo: edite el símbolo para cambiar su aspecto, o separe la instancia.
