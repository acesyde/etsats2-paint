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
pkg-no-variant = no describe ninguna variante.
pkg-empty-variant = la variante «{ $variant }» no tiene texturas.
pkg-duplicate-variant = la variante «{ $variant }» está definida dos veces.
pkg-duplicate-texture = la textura «{ $texture }» está definida dos veces.
pkg-bad-size = la textura «{ $texture }» tiene un tamaño no válido ({ $size }).
pkg-unsafe-path = contiene una ruta no segura ({ $path }).
pkg-too-large = es demasiado grande.
pkg-missing-template = falta la plantilla de «{ $texture }».
pkg-bad-template = la plantilla de «{ $texture }» no es una imagen PNG o SVG legible.
pkg-template-too-large = la plantilla de «{ $texture }» es demasiado grande.

## Vehicle library and updates

filter-packages = Paquetes de vehículos
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
vehicles-details = Versiones del juego { $versions } · { $variants }
vehicles-version = Versión { $version }
vehicles-remove-version = Eliminar { $name } { $version }
vehicles-remove-confirm = ¿Eliminar { $name } de la biblioteca? Los proyectos conservan sus plantillas.
vehicles-remove = Eliminar
vehicles-install = Instalar…
vehicles-install-sample = Instalar el vehículo de ejemplo
update-versions = { $name }: versión { $from } → { $to }
update-replaced = { $name }: plantilla reemplazada
update-layout-changed = { $name }: disposición cambiada, revise el diseño
update-resized = { $name }: tamaño { $old } → { $new } px, diseño escalado
update-added = { $name }: textura nueva
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
fleet-error-variants = Elija al menos una variante de este vehículo.
fleet-error-version = La versión instalada es distinta de la del proyecto.
fleet-error-unknown = Este vehículo no está en el proyecto.
variants-title = Variantes de { $name }
variants-missing-version = La versión { $version } de este vehículo no está instalada. Actualice su plantilla para cambiar sus variantes.
variants-remove-confirm = El diseño de { $variants } se eliminará con ellas.
variants-remove = Eliminar
variants-apply = Aplicar
remove-vehicle-confirm = ¿Quitar { $name } y su diseño del proyecto?
