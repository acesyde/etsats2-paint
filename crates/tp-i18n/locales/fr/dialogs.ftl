## Tailles d’export et erreurs

size-mb = { $value } Mo
size-kb = { $value } Ko
size-about = environ { $value }
export-stopped = l’export s’est arrêté de manière inattendue

## Enregistrement et ouverture des projets

file-save-failed-title = Enregistrement impossible
file-save-failed = Impossible d’enregistrer { $file }.
file-save-failed-reason = Impossible d’enregistrer { $file } ({ $reason }). Vos modifications sont toujours ouvertes.
file-open-failed-title = Impossible d’ouvrir le projet
file-restore-failed-title = Impossible de restaurer le projet
file-recovered-copy = La copie récupérée
file-not-a-project = { $file } n’est pas un projet TruckPaint.
file-damaged = { $file } est endommagé et ne peut pas être ouvert.
file-newer-version = { $file } a été créé avec une version plus récente de TruckPaint.
file-io = Impossible de lire ou d’écrire { $file } ({ $reason }).

## Boîtes de dialogue de fichiers

dialog-open-project = Ouvrir un projet
dialog-save-project = Enregistrer le projet
dialog-export-texture = Exporter la texture
filter-project = Projet TruckPaint
filter-images = Images

## Écran d’accueil

home-recovered = Projets récupérés
home-recovered-hint = TruckPaint s’est fermé de manière inattendue. Ces projets contenaient des modifications non enregistrées.
home-never-saved = Jamais enregistré
home-recovery-damaged = Copie de récupération endommagée
home-recovery-damaged-hint = Cette copie ne peut pas être restaurée.
home-recovered-item = { $name } récupéré
home-restore = Restaurer
home-discard = Ignorer
app-tagline = Éditeur de livrées pour Euro Truck Simulator 2 et American Truck Simulator
home-new-project = Nouveau projet
home-new-project-tip = Créer un nouveau projet de livrée ({ $shortcut })
home-preferences = Préférences
home-recent = Projets récents
home-recent-empty = Aucun projet récent
home-recent-empty-hint = Les projets que vous ouvrez apparaîtront ici. Commencez par créer un nouveau projet.
home-recent-missing-item = { $name } (fichier introuvable)
home-remove-recent = Retirer de la liste
home-file-not-found = Fichier introuvable
home-open-recent = Ouvrir { $path }
home-file-moved = Le fichier de ce projet a été déplacé ou supprimé.

## Boîtes de dialogue

unsaved-title = Enregistrer les modifications de « { $name } » avant de fermer ?
unsaved-hint = Vos modifications seront perdues si vous ne les enregistrez pas.
button-dont-save = Ne pas enregistrer
button-cancel = Annuler
button-ok = OK
button-create = Créer
button-done = Terminé
button-close = Fermer
new-project-step-name = Nom
new-project-name = Nom du projet
prefs-interface = Interface
prefs-ui-scale = Échelle de l’interface
prefs-text-size = Taille du texte
prefs-scale-hint = Les modifications s’appliquent immédiatement. 100 % suit le facteur d’échelle de votre écran.
prefs-canvas = Zone de travail
prefs-grid-spacing = Espacement de la grille
prefs-reset = Rétablir les valeurs par défaut
shortcuts-temporary-hand = Outil Main temporaire
shortcuts-hold-space = Maintenir Espace
about-version = Version { $version }
about-tagline = Éditeur de livrées pour Euro Truck Simulator 2 et American Truck Simulator.

## Boîte de dialogue Exporter la texture

export-done = { $file } exporté
export-failed-title = Export impossible
export-failed = Impossible d’écrire { $file } ({ $reason }).
export-preview = Aperçu de l’export
export-info = { $size } × { $size } px · { $format } · { $bytes }
export-progress = Export de { $file }…
export-start = Exporter…
export-format = Format
export-dds-bc3 = BC3 / DXT5 (recommandé)
export-dds-rgba = RGBA non compressé
export-dds-encoding = Encodage DDS
export-size = Taille
export-background = Arrière-plan
export-transparent = Transparent
export-background-color = Couleur d’arrière-plan
prefs-language = Langue
file-dev-format = { $file } utilise un format de développement que cette version ne peut pas ouvrir.

## Vehicle packages

pkg-install-failed = { $file } n’a pas pu être installé : { $reason }
pkg-no-library = aucun dossier de bibliothèque de véhicules n’est disponible.
pkg-io = il n’a pas pu être lu ou écrit ({ $reason }).
pkg-not-a-zip = ce n’est pas un paquet de véhicule.
pkg-no-manifest = il ne contient pas de manifeste vehicle.json.
pkg-bad-manifest = son manifeste est invalide ({ $reason }).
pkg-newer-format = il a été créé pour une version plus récente de TruckPaint.
pkg-bad-id = son identifiant « { $id } » est invalide.
pkg-no-main-texture = il n’a aucune texture principale.
pkg-bad-game-path = son chemin dans le jeu « { $path } » est invalide (mots en a–z, 0–9 et _ séparés par des points).
pkg-bad-game-id = l’identifiant du jeu « { $id } » est invalide (mots en a–z, 0–9 et _ séparés par des points).
pkg-missing-game-ids = « { $texture } » n’a aucun identifiant du jeu.
pkg-duplicate-part = deux textures ont l’identifiant « { $id } ».
pkg-duplicate-game-id = l’identifiant du jeu « { $id } » est listé deux fois.
pkg-bad-size = la texture « { $texture } » a une taille invalide ({ $size }).
pkg-unsafe-path = il contient un chemin non sûr ({ $path }).
pkg-too-large = il est trop volumineux.
pkg-missing-template = le gabarit de « { $texture } » est manquant.
pkg-bad-template = le gabarit de « { $texture } » n’est pas une image PNG ou SVG lisible.
pkg-template-too-large = le gabarit de « { $texture } » est trop grand.

## Custom vehicles

custom-build-failed = { $name } n’a pas pu être créé : { $reason }
custom-template-unconvertible = le gabarit de « { $texture } » ne peut pas être converti.
custom-file-not-image = { $file } ne peut pas être utilisé : ce n’est pas une image PNG, DDS ou SVG.
custom-file-unsupported-dds = { $file } ne peut pas être lu : sa compression DDS ({ $format }) n’est pas prise en charge. Enregistrez-le en BC1, BC2, BC3 ou RGB(A) non compressé.
custom-file-damaged = { $file } ne peut pas être lu : le fichier est endommagé ou incomplet.
custom-file-too-large = { $file } ne peut pas être utilisé : il dépasse { $size } px.
custom-file-io = { $file } ne peut pas être lu ({ $reason }).
custom-build-stopped = { $name } n’a pas pu être créé : la création s’est arrêtée de façon inattendue.
custom-title = Véhicule personnalisé
custom-new-version-title = Nouvelle version de { $name }
custom-open = Véhicule personnalisé…
custom-open-library = Véhicule personnalisé…
custom-id = Identifiant
custom-version = Version
custom-name = Nom
custom-brand = Marque
custom-kind = Type
custom-game = Jeu
custom-game-path = Chemin dans le jeu
custom-game-path-hint = Comme dans les définitions du jeu, p. ex. scania.r_2016
custom-game-versions = Versions du jeu
custom-game-versions-hint = Toutes les versions, ou une plage comme >=1.53
custom-alt-uv = Jeu d’UV alternatif
custom-colour-picker = Choix de couleur
custom-textures = Textures
custom-drop-hint = Déposez ici les gabarits du jeu (PNG, DDS ou SVG), ou ajoutez-les. Chaque fichier est une texture.
custom-add-templates = Ajouter des gabarits…
custom-row-name = Nom de la texture { $n }
custom-row-role = Rôle de la texture { $n }
custom-row-size = Taille de la texture { $n }
custom-row-game-ids = Identifiants de jeu de la texture { $n }
custom-row-replace = Remplacer…
custom-row-replace-name = Remplacer le gabarit de la texture { $n }
custom-row-remove = Supprimer la texture { $n }
custom-role-main = Texture principale
custom-role-accessory = Accessoire
custom-hint-cabins = Noms internes des cabines qui utilisent cette disposition, p. ex. highline, highline_8x4
custom-hint-cabins-optional = Noms internes des cabines qui utilisent cette disposition ; laissez vide pour peindre toutes les cabines
custom-hint-accessories = Identifiants des accessoires couverts par cette texture, p. ex. mirror.painted, s_mirror.painted
custom-file = { $file } · { $width } × { $height } px
custom-not-square = Pas carré : l’image sera étirée sur la texture carrée.
custom-scs-reminder = Les gabarits des jeux de base appartiennent à SCS Software : utilisez-les pour vos propres livrées, et vérifiez leur licence avant de partager un paquet.
custom-create = Créer
custom-building = Création du paquet… ({ $done }/{ $total })
custom-building-plain = Création du paquet…
custom-problem-name = Saisissez le nom du véhicule.
custom-problem-brand = Saisissez la marque du véhicule.
custom-problem-game-path = Le chemin dans le jeu est fait de mots en a–z, 0–9 et _ séparés par des points, comme dans les définitions du jeu, p. ex. scania.r_2016.
custom-problem-game-versions = Saisissez une plage de versions comme >=1.53, ou laissez vide pour toutes les versions.
custom-problem-version = Saisissez une version comme 1.1.0.
custom-problem-version-not-higher = La version doit être supérieure à { $version }.
custom-problem-installed = { $id } est déjà installé. Créez-en une nouvelle version avec Nouvelle version… dans la bibliothèque de véhicules, ou changez le nom.
custom-problem-no-main = Faites de l’une des textures une texture principale.
custom-problem-trailer-main = Une remorque a une seule texture principale : faites des autres des accessoires.
custom-problem-texture-name = Saisissez le nom de la texture.
custom-problem-cabins = Saisissez les noms internes des cabines qui utilisent cette disposition.
custom-problem-accessory-ids = Saisissez les identifiants des accessoires couverts par cette texture.
custom-problem-game-id = « { $id } » n’est pas un identifiant de jeu valide : utilisez des mots en a–z, 0–9 et _ séparés par des points.
custom-problem-duplicate-game-id = « { $id } » est déjà utilisé par une autre texture.

## Vehicle library and updates

filter-packages = Paquets de véhicules
filter-templates = Gabarits (PNG, DDS, SVG)
dialog-export-package = Exporter le paquet de véhicule (les gabarits des jeux de base appartiennent à SCS Software)
vehicles-search = Rechercher un véhicule
vehicles-all-games = Tous les jeux
vehicles-game-filter = Jeu
vehicles-all-kinds = Camions et remorques
vehicles-kind-filter = Type
vehicles-truck = Camion
vehicles-trailer = Remorque
vehicles-installed = { $name } { $version } installé
vehicles-empty = Aucun véhicule installé
vehicles-empty-hint = Les paquets de véhicules (.tpv) contiennent les gabarits d’un camion ou d’une remorque. Installez-en un pour y peindre une livrée.
vehicles-details = Versions du jeu { $versions } · { $main } · Accessoires : { $accessories }
vehicles-version = Version { $version }
vehicles-remove-version = Supprimer { $name } { $version }
vehicles-remove-confirm = Supprimer { $name } de la bibliothèque ? Les projets conservent leurs gabarits.
vehicles-remove = Supprimer
vehicles-install = Installer…
vehicles-install-sample = Installer les véhicules d’exemple
vehicles-any-version = toutes les versions
vehicles-new-version = Nouvelle version…
vehicles-new-version-of = Nouvelle version de { $name }
vehicles-export-version = Exporter { $name } { $version }
vehicles-exported = { $file } exporté
vehicles-export-failed = { $file } n’a pas pu être écrit : { $reason }
update-versions = { $name } : version { $from } → { $to }
update-replaced = { $name } : gabarit remplacé
update-layout-changed = { $name } : disposition modifiée, vérifiez le motif
update-resized = { $name } : taille { $old } → { $new } px, motif mis à l’échelle
update-new-textures = Nouveau dans cette version :
update-removed = { $name } : absente de cette version, motif conservé
update-artwork-kept = Votre motif est conservé. Vous pouvez annuler la mise à jour.
update-apply = Mettre à jour

## New Project wizard

new-project-step-vehicle = Véhicule
new-project-no-vehicles-hint = Pas encore de véhicule ? Essayez le camion d’exemple, ou installez un paquet.
new-project-textures = Textures
button-next = Suivant
button-back = Retour

## Fleet

add-vehicle-none = Aucun autre véhicule de ce jeu n’est installé. Installez un paquet pour en ajouter un.
add-vehicle-add = Ajouter
fleet-error-other-game = Ce véhicule est pour un autre jeu que le projet.
fleet-error-already-there = Ce véhicule est déjà dans le projet.
fleet-error-textures = Choisissez au moins une texture principale de ce véhicule.
fleet-error-version = La version installée diffère de celle du projet.
fleet-error-unknown = Ce véhicule n’est pas dans le projet.
textures-title = Textures de { $name }
textures-missing-version = La version { $version } de ce véhicule n’est pas installée. Mettez à jour son gabarit pour changer ses textures.
textures-remove-confirm = Le travail sur { $textures } sera supprimé avec elles.
textures-remove = Supprimer
textures-apply = Appliquer
remove-vehicle-confirm = Retirer { $name } et son travail du projet ?
