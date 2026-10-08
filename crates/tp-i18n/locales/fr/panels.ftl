## Titres des panneaux et états vides

panel-properties = Propriétés
panel-layers = Calques
panel-colors = Couleurs
panel-styles = Styles
panel-symbols = Symboles
panel-stroke = Contour
panel-transform = Transformation
panel-assets = Ressources
panel-vehicle = Véhicules
empty-properties = Aucune sélection
empty-properties-hint = Sélectionnez un objet dans la zone de travail pour modifier ses propriétés.
empty-layers = Aucun calque pour l’instant
empty-layers-hint = Les formes, textes et images que vous ajoutez apparaîtront ici.
empty-colors = Aucune couleur sélectionnée
empty-colors-hint = Choisissez un fond ou un contour pour modifier sa couleur.
empty-styles = Aucun style
empty-styles-hint = Enregistrez des apparences à réutiliser sur chaque texture.
empty-symbols = Aucun symbole
empty-symbols-hint = Convertissez un logo ou un lettrage en symbole pour le placer sur chaque texture.
empty-stroke = Aucun contour
empty-stroke-hint = Sélectionnez un objet pour modifier son contour.
empty-transform = Rien à transformer
empty-transform-hint = Sélectionnez un objet pour modifier sa position, sa taille et sa rotation.
empty-assets = Aucune ressource
empty-assets-hint = Les logos et images importés seront listés ici.
empty-vehicle = Aucun véhicule
empty-vehicle-hint = Les modèles de véhicules seront disponibles dans une prochaine mise à jour.

## Panneaux et calques

panels-all-closed = Tous les panneaux sont fermés
panels-all-closed-hint = Rouvrez les panneaux depuis le menu Affichage, ou réinitialisez l’espace de travail.
panel-expand = Déplier
panel-collapse = Replier
panel-close = Fermer le panneau
layers-empty-hint = Les formes que vous dessinez et les calques que vous ajoutez apparaissent ici.
layers-collapse = Replier { $name }
layers-expand = Déplier { $name }
layers-hide = Masquer { $name }
layers-show = Afficher { $name }
layers-lock = Verrouiller { $name }
layers-unlock = Déverrouiller { $name }
layers-name = Nom du calque

## Ressources

assets-uses = { $count ->
    [one] { $count } utilisation
   *[other] { $count } utilisations
    }
assets-used-by = { $count ->
    [one] Utilisée par { $count } objet
   *[other] Utilisée par { $count } objets
    }
assets-vector = Vectorielle
assets-item = Ressource { $name }
assets-place = Importer la ressource
assets-name = Nom de la ressource

## Panneau Contour

stroke-enabled = Contour activé
stroke-width = Épaisseur du contour
stroke-center = Centré
stroke-center-name = Contour centré
stroke-inside = Intérieur
stroke-inside-name = Contour intérieur
stroke-outside = Extérieur
stroke-outside-name = Contour extérieur
undo-change-stroke-width = Modifier l’épaisseur du contour
undo-change-stroke-alignment = Modifier l’alignement du contour
undo-add-stroke = Ajouter un contour
undo-remove-stroke = Supprimer le contour

## Panneau Transformation

field-width = Largeur
field-height = Hauteur
field-w = L
field-h = H
field-r = R
field-s = É
transform-x = Position X
transform-y = Position Y
transform-rotation = Rotation
transform-scale = Échelle
transform-unlock-proportions = Déverrouiller les proportions
transform-lock-proportions = Verrouiller les proportions
transform-align-to = Aligner sur

## Panneau Propriétés

undo-change-opacity = Modifier l’opacité
undo-change-corner-radius = Modifier le rayon des angles
undo-change-sides = Modifier le nombre de côtés
undo-change-inner-radius = Modifier le rayon intérieur
undo-change-line-width = Modifier l’épaisseur de ligne
undo-change-star = Modifier l’étoile
props-sides = Côtés
props-star = Étoile
props-inner = Intérieur
props-inner-radius = Rayon intérieur
props-surface-summary = { $size } × { $size } px · Sélectionnez un objet pour le modifier
props-objects = { $count ->
    [one] { $count } objet
   *[other] { $count } objets
    }
props-multiple = Sélection multiple
props-source-svg = SVG · Vectoriel
props-source-image = Image · { $width } × { $height } px
props-source = Source : { $name } · { $source }
props-opacity = Opacité
props-opacity-slider = Curseur d’opacité
props-radius = Rayon
props-corner-radius = Rayon des angles
props-line-width = Épaisseur de ligne
props-fill = Fond
props-fill-color = Couleur du fond
props-stroke-color = Couleur du contour

## Pointillés, extrémités et angles

line-preset-dashed = Tirets 20/10
line-preset-dotted = Points 0/12
line-preset-long-dash = Tirets longs 60/20
undo-change-stroke-caps = Modifier les extrémités du contour
undo-change-stroke-joins = Modifier les angles du contour
undo-change-stroke-dashes = Modifier les pointillés du contour
undo-change-line-caps = Modifier les extrémités de la ligne
undo-change-line-joins = Modifier les angles de la ligne
undo-change-line-dashes = Modifier les pointillés de la ligne
line-dashed = Pointillés
line-presets = Préréglages
line-dash-presets = Préréglages de pointillés
line-dash = Tiret
line-dash-length = Longueur du tiret
line-gap = Espace
line-gap-length = Longueur de l’espace
line-cap = Extrémité
line-butt = Plat
line-butt-cap = Extrémité plate
line-round = Arrondi
line-round-cap = Extrémité arrondie
line-square = Carré
line-square-cap = Extrémité carrée
line-join = Angle
line-miter = Pointe
line-miter-join = Angle en pointe
line-round-join = Angle arrondi
line-bevel = Biseauté
line-bevel-join = Angle biseauté
line-limit = Limite
line-miter-limit = Limite de pointe

## Panneau Couleurs

colors-no-stroke = Aucun contour
colors-solid = Uni
colors-solid-paint = Couleur unie
colors-linear = Linéaire
colors-linear-gradient = Dégradé linéaire
colors-radial = Radial
colors-radial-gradient = Dégradé radial
colors-location = Position
colors-stop-location = Position du point
colors-angle = Angle
colors-gradient-angle = Angle du dégradé
colors-reverse = Inverser le dégradé
colors-red = Rouge
colors-green = Vert
colors-blue = Bleu
colors-hue = Teinte
colors-saturation = Saturation
colors-value = Valeur
colors-lightness = Luminosité
colors-alpha = Alpha
colors-hex = Hex
colors-hex-color = Couleur hexadécimale
colors-hex-invalid = Couleur hexadécimale non valide. Utilisez #RGB, #RRGGBB ou #RRGGBBAA.
colors-recent = Récentes
colors-recent-item = Couleur récente { $color }
colors-palette = Palette
colors-add-to-palette = Ajouter à la palette
colors-differ = Les couleurs diffèrent dans la sélection.
colors-palette-empty = Enregistrez les couleurs que vous réutilisez avec +.
colors-swatch-prefix = Couleur
colors-edit-swatch = Modifier la nuance…
colors-delete-swatch = Supprimer la nuance
colors-linked-to = Liée à { $name }
colors-swatch-name = Nom
colors-swatch-name-empty = Saisissez un nom pour la nuance.
colors-swatch-hex = Couleur hexadécimale de la nuance
colors-aspect = Proportions
colors-aspect-ratio = Proportions du dégradé

## Panneau Caractère

weight-thin = Fin
weight-extra-light = Extra-léger
weight-light = Léger
weight-regular = Normal
weight-medium = Moyen
weight-semi-bold = Demi-gras
weight-bold = Gras
weight-extra-bold = Extra-gras
weight-black = Noir
char-title = Caractère
char-font-weight = Graisse de la police
char-italic = Italique
char-size = Taille
char-font-size = Taille de la police
char-align-left = Aligner à gauche
char-align-center = Centrer
char-align-right = Aligner à droite
char-tracking = Approche
char-letter-spacing = Espacement des lettres
char-line = Interligne
char-line-height = Hauteur de ligne
char-family-missing-name = Famille de polices : { $family } (police introuvable : { $family })
char-family-name = Famille de polices : { $family }
char-font-not-found = Police introuvable : { $family }
char-font-family = Famille de polices
char-search-fonts = Rechercher des polices
char-no-fonts = Aucune police correspondante
undo-change-font-weight = Modifier la graisse
undo-change-italic = Modifier l’italique
undo-change-text-size = Modifier la taille du texte
undo-change-alignment = Modifier l’alignement
undo-change-letter-spacing = Modifier l’espacement des lettres
undo-change-line-height = Modifier la hauteur de ligne
undo-change-font = Modifier la police

## Vehicle panel

vehicle-panel-package = Paquet { $version } · versions du jeu { $games }
vehicle-panel-package-missing = Paquet { $version } (non installé)
vehicle-panel-update = La version { $version } est disponible
vehicle-panel-texture = Texture { $name }
vehicle-panel-layout-changed = Disposition modifiée
vehicle-panel-dismiss = Ignorer
vehicle-panel-dismiss-named = Ignorer le changement de disposition de { $name }
vehicle-panel-removed = Absente de cette version
vehicle-panel-opacity = Gabarit
vehicle-panel-opacity-name = Opacité du gabarit

## Fleet

vehicle-panel-update-named = Mettre à jour le gabarit de { $name }
vehicles-textures = Textures…
vehicles-main-textures = Textures principales
vehicles-accessories = Accessoires
vehicles-remove-from-project = Retirer du projet

## Vehicles sidebar


## Sidebar

sidebar-project = Projet
sidebar-show = Afficher la barre latérale
sidebar-hide = Masquer la barre latérale
sidebar-vehicles-game = Véhicules · { $game }
project-name = Nom
project-version = Version
project-game-versions = Versions du jeu
project-game-versions-supported = Prises en charge par tous les véhicules : { $versions }
project-no-common-version = Aucune version du jeu n’est prise en charge par tous les véhicules
vehicle-actions-named = Actions pour { $name }
vehicle-panel-needs-check = Une texture est à vérifier après une mise à jour
texture-layout-changed = La disposition de cette texture a changé en version { $version } : vérifiez votre dessin.
texture-not-in-version = Cette texture n’existe pas dans la version { $version } du véhicule : elle n’a pas de gabarit.

## Styles panel

styles-graphic = Styles graphiques
styles-text = Styles de texte
styles-graphic-empty = Enregistrez un fond, un contour et une opacité à réutiliser sur chaque texture : sélectionnez une forme et cliquez sur +.
styles-text-empty = Enregistrez des réglages de lettrage à réutiliser sur chaque texture : sélectionnez un texte et cliquez sur +.
styles-new-graphic = Nouveau style graphique d’après la sélection
styles-new-text = Nouveau style de texte d’après la sélection
styles-new = Nouveau style
styles-need-shape = Sélectionnez une forme ou un texte.
styles-need-text = Sélectionnez un texte.
styles-graphic-item = Style graphique { $name }
styles-text-item = Style de texte { $name }
styles-graphic-prefix = Style
styles-text-prefix = Style de texte
styles-apply = Appliquer le style
styles-rename = Renommer
styles-redefine = Redéfinir d’après la sélection
styles-select-users = Sélectionner les utilisateurs sur cette texture
styles-delete = Supprimer le style
styles-name = Nom du style

## Symbols panel

symbols-prefix = Symbole
symbols-copy-suffix = copie
symbols-empty = Dessinez une fois un logo ou un lettrage, sélectionnez-le et choisissez Convertir en symbole : placez-le sur chaque texture, et modifiez-le une fois pour toutes.
symbols-instances = { $count ->
    [one] { $count } instance
   *[other] { $count } instances
    }
symbols-item = Symbole { $name }
symbols-place = Placer { $name }
symbols-edit = Modifier { $name }
symbols-rename = Renommer
symbols-duplicate = Dupliquer
symbols-delete = Supprimer le symbole
symbols-delete-confirm = Supprimer { $name } ? { $count ->
    [one] Son instance devient un groupe à l’identique.
   *[other] Ses { $count } instances deviennent des groupes à l’identique.
    }
symbols-name = Nom du symbole
symbol-bar-editing = Modification du symbole { $name }
symbol-bar-done = Terminé
props-instance-of = Instance de { $name }
instance-look-in-symbol = Une instance montre son symbole : modifiez le symbole pour changer son apparence, ou détachez l’instance.

## Library

library-add = Ajouter à la bibliothèque
library-update = Mettre à jour dans la bibliothèque
library-added = Ajouté à la bibliothèque
library-updated = Bibliothèque mise à jour
undo-import-from-library = Importer depuis la bibliothèque
