# Générer les véhicules TruckPaint depuis le jeu

Notes de décision et prochaines étapes — 12 octobre 2026.

## Objectif

Construire les packages `.tpv` des camions et remorques directement depuis
l'installation ETS2 ou ATS du joueur. Chaque package contient la description
du véhicule, ses variantes et les gabarits des textures à peindre.

Le format `.tpv` reste pertinent : une fois construit, le package est autonome
et les projets embarquent leurs gabarits. C'est sa chaîne de fabrication que
l'on veut rendre indépendante de Carsmaniac et de Paintjob Packer.

## Pourquoi changer la source

Le catalogue actuel décrit les véhicules avec des données de Paintjob Packer
et un mapping vers les dossiers et images du pack Carsmaniac. La branche
`origin/feat/official-vehicles-ets2` étend cette approche à 66 véhicules ETS2
et ajoute `tpv catalog match` pour proposer les correspondances.

Ce travail est utile pour couvrir rapidement les véhicules, mais dépend des
choix d'organisation, des mises à jour et de la disponibilité d'un tiers.
Un changement de noms ou de regroupements dans le pack peut casser l'import
sans que le véhicule ait changé dans le jeu.

Direction retenue : utiliser les données du jeu comme référence. Le JSON
décrivant un véhicule doit devenir, autant que possible, un résultat généré
et versionné. Le mapping Carsmaniac peut rester un adaptateur d'import
facultatif. Le travail de la branche reste utile pour comparer les résultats
et pour les parcours de sélection, d'installation et d'export existants.

## Ce que les essais ont démontré

Les essais utilisent une installation ETS2 **1.61.1.1**. Le pack Carsmaniac
local porte la version **1.59** : la comparaison visuelle ne porte donc pas
sur la même version du jeu.

| Véhicule | Chemin du jeu | Cabines | Accessoires indépendants | Modèles convertis |
|---|---|---|---:|---:|
| MAN TGX récent | `man.tgx_2020` | GX, GM, GN | 27 | 195 |
| DAF NGD | `daf.2021` | XG+, XG, XF | 12 | 172 |

Pour chacun, le prototype a :

1. extrait les définitions du jeu et les identifiants de cabines/accessoires ;
2. suivi les références des modèles, looks et variantes ;
3. converti les modèles avec ConverterPIX ;
4. sélectionné les faces visibles portant les matériaux de peinture ;
5. généré des gabarits SVG depuis leurs UV et un `vehicle.json` ;
6. construit le `.tpv`, un projet `.truckpaint` avec grilles numérotées et
   un mod `.scs` de contrôle avec l'exporteur existant de TruckPaint.

**La génération ne lit ni les données Paintjob Packer ni les images
Carsmaniac.** Les images Carsmaniac ont uniquement servi à comparer
visuellement les résultats après extraction.

Les tests ciblés, la validation native des packages, la sauvegarde/relecture
des projets et les vérifications des textures et références des mods ont
réussi. L'utilisateur a ensuite testé les véhicules en jeu et confirmé que
les configurations vérifiées étaient bonnes, pour le MAN puis pour le DAF.
Cela valide la faisabilité sur deux modèles distincts ; aucune matrice
exhaustive de toutes les configurations n'a été enregistrée.

La lecture directe du MAN a aussi révélé des écarts avec le catalogue de la
branche : des identifiants de jupes inversés (`4x2_p.sideskirt` contre
`sideskirt.4x2_p` dans les définitions installées) et la variante caméra GN
`mirror.cam_pnt_gn` absente. Une comparaison avec le jeu est donc utile même
si l'on conserve un import de templates externes.

## Limites du prototype

- Seuls les deux profils MAN et DAF sont décrits ; ce n'est pas encore un
  extracteur universel ni une fonctionnalité intégrée à l'application.
- Les modèles extérieurs sont couverts. Les autres DLC et les mods qui
  remplacent les véhicules ne sont pas montés par ces essais.
- Les matériaux examinés utilisent le shader de peinture ordinaire
  `eut2.truckpaint.tsnmapuv`. Le flux portant l'alias `_TEXCOORD1` a fonctionné
  sur les deux véhicules. Ne pas généraliser cette règle à tous les shaders.
- Les UV alternatifs sont actuellement refusés.
- Les coordonnées produites par ConverterPIX sont déjà en coordonnées
  d'image : le prototype n'ajoute pas de retournement vertical.
- Les gabarits principaux superposent les options peintes compatibles ;
  certains traits se chevauchent.
- Un accessoire indépendant correspond à une surface distincte, sans
  regroupement ergonomique ; les noms restent techniques.
- Les cabines sont proposées en 4096 × 4096, par choix de résolution de
  sortie. Les dimensions des accessoires sont déduites des noms des textures
  découpées des peintures SCS ; cette méthode reste à vérifier sur les
  prochains véhicules.
- ConverterPIX reste une dépendance technique pour lire les formats binaires.
  L'indépendance visée porte sur la source des données et des gabarits.

## Suite proposée, dans l'ordre

### 1. Couvrir les cas encore différents

Prochain essai recommandé : **une remorque avec plusieurs carrosseries ou
longueurs**. Vérifier les textures principales, les corps et les accessoires,
en conservant le format `.tpv` existant si possible.

Ensuite, tester **un camion avec UV alternatifs**, notamment un cas ATS.
Identifier le mapping utilisé par le jeu, produire les gabarits correspondants
et vérifier que l'export choisit le même jeu d'UV.

Pour chaque essai : package, projet de contrôle, mod numéroté, vérifications
des fichiers et validation en jeu. Corriger les règles à partir des écarts
observés avant de traiter une flotte entière.

### 2. Fiabiliser l'extracteur

À partir des cas validés, rendre la chaîne reproductible : installation du
jeu → définitions et modèles → gabarits et manifeste → `.tpv` validé.

Remplacer les hypothèses propres aux profils par des informations du jeu
lorsqu'elles sont disponibles. Signaler les modèles, shaders, dimensions ou
correspondances non pris en charge, avec leurs sources, plutôt que produire
silencieusement un package incomplet. Conserver la version du jeu et les
sources de chaque texture pour pouvoir comprendre une mise à jour.

### 3. Étendre les véhicules officiels par lots

Traiter les véhicules ETS2 puis ATS par familles de cas déjà validés.
Comparer avec le catalogue existant pour repérer les oublis et différences,
mais sans en faire une dépendance de génération. Produire un mod de contrôle
par lot et enregistrer les configurations testées ainsi que les limites.

### 4. Intégrer dans TruckPaint

Le joueur indique son installation du jeu, choisit les véhicules à préparer,
et l'application construit et installe les `.tpv` localement. Réutiliser les
parcours existants de bibliothèque, de projets et d'export.

Après une mise à jour du jeu, comparer les layouts générés, signaler les
textures qui ont changé et réutiliser le mécanisme de version et de mise à
jour des templates. Carsmaniac reste une source d'import possible, sans
conditionner cette génération locale.

## Où retrouver les essais

- [Extracteur partagé](../experiments/man-tgx-game/extract.py).
- [Tests ciblés](../experiments/man-tgx-game/test_extract.py).
- [Construction des projets et mods avec le code TruckPaint](../crates/tp-app/examples/man_tgx_game_probe.rs).
- [Procédure et fichiers MAN](../experiments/man-tgx-game/README.md).
- [Procédure et fichiers DAF](../experiments/man-tgx-game/DAF-NGD.md).

Les rapports et fichiers générés sont sous
`target/prototypes/man-tgx-game/run-1/` et
`target/prototypes/daf-ngd-game/run-1/`. Ils sont locaux et ignorés par Git.
Les mentions « validation en jeu en attente » des documents et rapports
initiaux décrivent leur état à la génération ; les confirmations utilisateur
ultérieures sont consignées dans la présente note.

**Prochaine action : choisir une remorque à plusieurs carrosseries et refaire
le même essai complet.**
