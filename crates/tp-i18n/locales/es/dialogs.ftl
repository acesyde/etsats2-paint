## Tamaños y errores de exportación

size-mb = { $value } MB
size-kb = { $value } KB
size-about = unos { $value }
export-stopped = la exportación se detuvo inesperadamente

## Guardar y abrir proyectos

file-save-failed-title = No se ha podido guardar
file-save-failed = No se ha podido guardar { $file }.
file-save-failed-reason = No se ha podido guardar { $file } ({ $reason }). Sus cambios siguen abiertos.
file-open-failed-title = No se puede abrir el proyecto
file-restore-failed-title = No se puede restaurar el proyecto
file-recovered-copy = La copia recuperada
file-not-a-project = { $file } no es un proyecto de TruckPaint.
file-damaged = { $file } está dañado y no se puede abrir.
file-newer-version = { $file } se creó con una versión más reciente de TruckPaint.
file-io = No se ha podido leer ni escribir { $file } ({ $reason }).

## Diálogos de archivo

dialog-open-project = Abrir proyecto
dialog-save-project = Guardar proyecto
dialog-export-texture = Exportar textura
filter-project = Proyecto de TruckPaint
filter-images = Imágenes

## Pantalla de inicio

home-recovered = Proyectos recuperados
home-recovered-hint = TruckPaint se cerró inesperadamente. Estos proyectos tenían cambios sin guardar.
home-never-saved = Nunca guardado
home-recovery-damaged = Copia de recuperación dañada
home-recovery-damaged-hint = Esta copia no se puede restaurar.
home-recovered-item = { $name } recuperado
home-restore = Restaurar
home-discard = Descartar
app-tagline = Editor de libreas para Euro Truck Simulator 2 y American Truck Simulator
home-new-project = Nuevo proyecto
home-new-project-tip = Crear un nuevo proyecto de librea ({ $shortcut })
home-preferences = Preferencias
home-recent = Proyectos recientes
home-recent-empty = No hay proyectos recientes
home-recent-empty-hint = Los proyectos que abra aparecerán aquí. Empiece creando un nuevo proyecto.
home-recent-missing-item = { $name } (archivo no encontrado)
home-remove-recent = Quitar de la lista
home-file-not-found = Archivo no encontrado
home-open-recent = Abrir { $path }
home-file-moved = El archivo de este proyecto se movió o se eliminó.

## Diálogos

unsaved-title = ¿Guardar los cambios de «{ $name }» antes de cerrar?
unsaved-hint = Sus cambios se perderán si no los guarda.
button-dont-save = No guardar
button-cancel = Cancelar
button-ok = Aceptar
button-create = Crear
button-done = Listo
button-close = Cerrar
new-project-step-name = Nombre
new-project-name = Nombre del proyecto
prefs-interface = Interfaz
prefs-ui-scale = Escala de la interfaz
prefs-text-size = Tamaño del texto
prefs-scale-hint = Los cambios se aplican de inmediato. 100 % sigue el factor de escala de su pantalla.
prefs-canvas = Lienzo
prefs-grid-spacing = Espaciado de la cuadrícula
prefs-reset = Restablecer valores predeterminados
shortcuts-temporary-hand = Herramienta Mano temporal
shortcuts-hold-space = Mantener pulsada la barra espaciadora
about-version = Versión { $version }
about-tagline = Editor de libreas para Euro Truck Simulator 2 y American Truck Simulator.

## Diálogo Exportar textura

export-done = { $file } exportado
export-failed-title = No se ha podido exportar
export-failed = No se ha podido escribir { $file } ({ $reason }).
export-preview = Vista previa de la exportación
export-info = { $size } × { $size } px · { $format } · { $bytes }
export-progress = Exportando { $file }…
export-start = Exportar…
export-format = Formato
export-dds-bc3 = BC3 / DXT5 (recomendado)
export-dds-rgba = RGBA sin comprimir
export-dds-encoding = Codificación DDS
export-size = Tamaño
export-background = Fondo
export-transparent = Transparente
export-background-color = Color de fondo
prefs-language = Idioma
file-dev-format = { $file } usa un formato de desarrollo que esta versión no puede abrir.

## Vehicle packages

pkg-install-failed = No se pudo instalar { $file }: { $reason }
pkg-no-library = no hay carpeta de biblioteca de vehículos.
pkg-io = no se pudo leer ni escribir ({ $reason }).
pkg-not-a-zip = no es un paquete de vehículo.
pkg-no-manifest = no contiene el manifiesto vehicle.json.
pkg-bad-manifest = su manifiesto no es válido ({ $reason }).
pkg-newer-format = se creó para una versión más reciente de TruckPaint.
pkg-bad-id = su identificador «{ $id }» no es válido.
pkg-no-main-texture = no tiene ninguna textura principal.
pkg-bad-game-path = su ruta en el juego «{ $path }» no es válida (palabras de a–z, 0–9 y _ separadas por puntos).
pkg-bad-game-id = el identificador del juego «{ $id }» no es válido (palabras de a–z, 0–9 y _ separadas por puntos).
pkg-missing-game-ids = «{ $texture }» no tiene ningún identificador del juego.
pkg-duplicate-part = dos texturas tienen el identificador «{ $id }».
pkg-duplicate-game-id = el identificador del juego «{ $id }» aparece dos veces.
pkg-bad-size = la textura «{ $texture }» tiene un tamaño no válido ({ $size }).
pkg-unsafe-path = contiene una ruta no segura ({ $path }).
pkg-too-large = es demasiado grande.
pkg-missing-template = falta la plantilla de «{ $texture }».
pkg-bad-template = la plantilla de «{ $texture }» no es una imagen PNG o SVG legible.
pkg-template-too-large = la plantilla de «{ $texture }» es demasiado grande.

## Custom vehicles

custom-build-failed = No se pudo crear { $name }: { $reason }
custom-template-unconvertible = la plantilla de «{ $texture }» no se puede convertir.
custom-file-not-image = { $file } no se puede usar: no es una imagen PNG, DDS o SVG.
custom-file-unsupported-dds = { $file } no se puede leer: su compresión DDS ({ $format }) no es compatible. Guárdelo como BC1, BC2, BC3 o RGB(A) sin comprimir.
custom-file-damaged = { $file } no se puede leer: el archivo está dañado o incompleto.
custom-file-too-large = { $file } no se puede usar: supera los { $size } px.
custom-file-io = { $file } no se puede leer ({ $reason }).
custom-build-stopped = No se pudo crear { $name }: la creación se detuvo de forma inesperada.
custom-title = Vehículo personalizado
custom-new-version-title = Nueva versión de { $name }
custom-open = Vehículo personalizado…
custom-open-library = Vehículo personalizado…
custom-id = Identificador
custom-version = Versión
custom-name = Nombre
custom-brand = Marca
custom-kind = Tipo
custom-game = Juego
custom-game-path = Ruta en el juego
custom-game-path-hint = Como en las definiciones del juego, p. ej. scania.r_2016
custom-game-versions = Versiones del juego
custom-game-versions-hint = Cualquier versión, o un rango como >=1.53
custom-alt-uv = Conjunto UV alternativo
custom-colour-picker = Selector de color
custom-textures = Texturas
custom-drop-hint = Suelte aquí las plantillas del juego (PNG, DDS o SVG), o añádalas. Cada archivo es una textura.
custom-add-templates = Añadir plantillas…
custom-row-name = Nombre de la textura { $n }
custom-row-role = Función de la textura { $n }
custom-row-size = Tamaño de la textura { $n }
custom-row-game-ids = Identificadores de juego de la textura { $n }
custom-row-replace = Reemplazar…
custom-row-replace-name = Reemplazar la plantilla de la textura { $n }
custom-row-remove = Eliminar la textura { $n }
custom-role-main = Textura principal
custom-role-accessory = Accesorio
custom-hint-cabins = Nombres internos de las cabinas que usan esta disposición, p. ej. highline, highline_8x4
custom-hint-cabins-optional = Nombres internos de las cabinas que usan esta disposición; déjelo vacío para pintar todas las cabinas
custom-hint-accessories = Identificadores de los accesorios que cubre esta textura, p. ej. mirror.painted, s_mirror.painted
custom-file = { $file } · { $width } × { $height } px
custom-not-square = No es cuadrada: la imagen se estirará a la textura cuadrada.
custom-scs-reminder = Las plantillas de los juegos base pertenecen a SCS Software: úselas para sus propias libreas y compruebe su licencia antes de compartir un paquete.
custom-create = Crear
custom-building = Creando el paquete… ({ $done }/{ $total })
custom-building-plain = Creando el paquete…
custom-problem-name = Escriba el nombre del vehículo.
custom-problem-brand = Escriba la marca del vehículo.
custom-problem-game-path = La ruta en el juego son palabras de a–z, 0–9 y _ separadas por puntos, como en las definiciones del juego, p. ej. scania.r_2016.
custom-problem-game-versions = Escriba un rango de versiones como >=1.53, o déjelo vacío para cualquier versión.
custom-problem-version = Escriba una versión como 1.1.0.
custom-problem-version-not-higher = La versión debe ser superior a { $version }.
custom-problem-installed = { $id } ya está instalado. Cree una nueva versión con Nueva versión… en la biblioteca de vehículos, o cambie el nombre.
custom-problem-no-main = Convierta una de las texturas en textura principal.
custom-problem-trailer-main = Un remolque tiene una sola textura principal: convierta las demás en accesorios.
custom-problem-texture-name = Escriba el nombre de la textura.
custom-problem-cabins = Escriba los nombres internos de las cabinas que usan esta disposición.
custom-problem-accessory-ids = Escriba los identificadores de los accesorios que cubre esta textura.
custom-problem-game-id = «{ $id }» no es un identificador de juego válido: use palabras de a–z, 0–9 y _ separadas por puntos.
custom-problem-duplicate-game-id = «{ $id }» ya lo usa otra textura.

## Vehicle library and updates

filter-packages = Paquetes de vehículos
filter-templates = Plantillas (PNG, DDS, SVG)
dialog-export-package = Exportar paquete de vehículo (las plantillas de los juegos base pertenecen a SCS Software)
vehicles-search = Buscar vehículos
vehicles-all-games = Todos los juegos
vehicles-game-filter = Juego
vehicles-all-kinds = Camiones y remolques
vehicles-kind-filter = Tipo
vehicles-truck = Camión
vehicles-trailer = Remolque
vehicles-installed = { $name } { $version } instalado
vehicles-empty = Ningún vehículo instalado
vehicles-empty-hint = Los paquetes de vehículos (.tpv) contienen las plantillas de un camión o remolque. Instale uno para pintar una librea.
vehicles-details = Versiones del juego { $versions } · { $main } · Accesorios: { $accessories }
vehicles-version = Versión { $version }
vehicles-remove-version = Eliminar { $name } { $version }
vehicles-remove-confirm = ¿Eliminar { $name } de la biblioteca? Los proyectos conservan sus plantillas.
vehicles-remove = Eliminar
vehicles-install = Instalar…
vehicles-install-sample = Instalar los vehículos de ejemplo
vehicles-any-version = cualquier versión
vehicles-new-version = Nueva versión…
vehicles-new-version-of = Nueva versión de { $name }
vehicles-export-version = Exportar { $name } { $version }
vehicles-exported = { $file } exportado
vehicles-export-failed = No se pudo escribir { $file }: { $reason }
update-versions = { $name }: versión { $from } → { $to }
update-replaced = { $name }: plantilla reemplazada
update-layout-changed = { $name }: disposición cambiada, revise el diseño
update-resized = { $name }: tamaño { $old } → { $new } px, diseño escalado
update-new-textures = Nuevo en esta versión:
update-removed = { $name }: ya no está en esta versión, diseño conservado
update-artwork-kept = Su diseño se conserva. Puede deshacer la actualización.
update-apply = Actualizar

## New Project wizard

new-project-step-vehicle = Vehículo
new-project-no-vehicles-hint = ¿Aún sin vehículos? Pruebe el camión de ejemplo o instale un paquete.
new-project-textures = Texturas
button-next = Siguiente
button-back = Atrás

## Fleet

add-vehicle-none = No hay otro vehículo de este juego instalado. Instale un paquete para añadir uno.
add-vehicle-add = Añadir
fleet-error-other-game = Este vehículo es de otro juego que el proyecto.
fleet-error-already-there = Este vehículo ya está en el proyecto.
fleet-error-textures = Elija al menos una textura principal de este vehículo.
fleet-error-version = La versión instalada es distinta de la del proyecto.
fleet-error-unknown = Este vehículo no está en el proyecto.
textures-title = Texturas de { $name }
textures-missing-version = La versión { $version } de este vehículo no está instalada. Actualice su plantilla para cambiar sus texturas.
textures-remove-confirm = El diseño de { $textures } se eliminará con ellas.
textures-remove = Eliminar
textures-apply = Aplicar
copy-cabin-hint = Copia todos los objetos de la cabina elegida en { $texture }, en las mismas posiciones.
copy-cabin-objects = { $count ->
    [one] { $count } objeto
   *[other] { $count } objetos
 }
copy-cabin-empty = Esta textura no contiene ningún objeto que copiar.
copy-cabin-copy = Copiar
remove-vehicle-confirm = ¿Quitar { $name } y su diseño del proyecto?

# Export Mod
dialog-export-mod = Exportar mod
filter-mods = Mods del juego
filter-mod-images = Imágenes (PNG, JPEG)
mod-name = Nombre
mod-version = Versión
mod-author = Autor
mod-description = Descripción
mod-price = Precio
mod-unlock = Nivel de desbloqueo
mod-internal-name = Nombre interno
mod-internal-name-help = Nombra la pintura en el juego: a–z, 0–9 y _. Elija uno único.
mod-icon = Icono de la tienda
mod-image = Imagen del gestor de mods
mod-choose-icon = Elegir icono…
mod-generated-icon = Usar el icono generado
mod-choose-image = Elegir imagen…
mod-generated-image = Usar la imagen generada
mod-picture-failed = No se puede usar { $file } ({ $reason }).
mod-summary = En el mod
mod-summary-cabins = { $texture }: cabinas { $cabins }
mod-summary-every-cabin = { $texture }: todas las cabinas
mod-summary-not-painted = { $texture }: sin pintar
mod-summary-accessories = Accesorios: { $accessories }
mod-problem-name-empty = El mod necesita un nombre.
mod-problem-name-invalid = El nombre no puede contener ", \ ni saltos de línea.
mod-problem-version-invalid = La versión no puede contener ", \ ni saltos de línea.
mod-problem-author-invalid = El autor no puede contener ", \ ni saltos de línea.
mod-problem-internal-empty = El nombre interno no puede estar vacío.
mod-problem-internal-invalid = El nombre interno solo puede usar a–z, 0–9 y _.
mod-problem-internal-long = El nombre interno puede tener como máximo { $max } caracteres.
mod-problem-price = El precio debe ser mayor que 0.
mod-problem-same-path = { $first } y { $second } son el mismo vehículo en el juego ({ $path }): quite uno.
mod-problem-game-data = Hay que instalar { $vehicle } { $version } para exportar el mod.
mod-problem-bad-game-version = { $version } no es una versión del juego: escríbala como 1.56.* o 1.56.2.
mod-problem-unsupported-game-version = { $vehicle } ({ $range }) no es compatible con la versión del juego { $version }.
mod-problem-no-common-game-version = { $first } ({ $first_range }) y { $second } ({ $second_range }) no tienen ninguna versión del juego en común: actualice o quite uno de ellos.
mod-export-done = Mod exportado a { $file }
