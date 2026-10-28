# Το kneeboard (γραμματοσειρά γραμμών), ο σχεδιαστής αποστολής (src/planner.rs) και οι
# αποθηκευμένες αποστολές (src/savegame.rs). Οι συντομογραφίες πλοήγησης (PT, BRG, ETA, PRI,
# SEC, KM, KT) μένουν στα αγγλικά.

kb-brief = BRIEF
kb-route = ΔΙΑΔΡΟΜΗ
kb-map = ΧΑΡΤΗΣ
kb-target = ΣΤΟΧΟΣ
kb-keys = ΠΛΗΚΤΡΑ
kb-tip = Χάρτες, σημειώσεις αποστολής και οδηγός πλήκτρων
kb-geran-file = GERAN-2 (ΦΑΚΕΛΟΣ)
kb-objectives = ΣΤΟΧΟΙ
kb-primary = ΚΥΡΙΟΣ: { $objective }
kb-secondary = ΔΕΥΤΕΡΕΥΩΝ: { $objective }
kb-times = ΩΡΕΣ
kb-deadline = ΠΡΟΘΕΣΜΙΑ { $time }
kb-now = ΤΩΡΑ { $time }
kb-conditions = ΣΥΝΘΗΚΕΣ
kb-moon = ΣΕΛΗΝΗ: { $phase } ({ $lit }% ΦΩΤΙΣΜΕΝΗ)
kb-armament = ΟΠΛΙΣΜΟΣ
kb-roe = ΚΑΝΟΝΕΣ ΕΜΠΛΟΚΗΣ
kb-total = ΣΥΝΟΛΟ { $km } KM ΣΕ { $kt } KT
kb-fuel = ΚΑΥΣΙΜΟ { $lbs } LBS
kb-preparing = ΠΡΟΕΤΟΙΜΑΣΙΑ...
kb-airborne = ΣΤΟΝ ΑΕΡΑ
kb-developing = ΕΠΕΞΕΡΓΑΣΙΑ...
kb-no-photo = ΧΩΡΙΣ ΦΩΤΟΓΡΑΦΙΑ
kb-airborne-target = ΑΕΡΟΜΕΤΑΦΕΡΟΜΕΝΟΣ ΣΤΟΧΟΣ
kb-scroll-vr = ΚΥΛΙΣΗ: THUMBSTICK, ΔΑΧΤΥΛΟ ΣΤΟ ΠΙΝΑΚΑ
kb-scroll-desktop = ΚΥΛΙΣΗ: ΡΟΔΑ ΠΟΝΤΙΚΙΟΥ ΠΑΝΩ ΣΤΟΝ ΠΙΝΑΚΑ

## Σχεδιαστής αποστολής

planner-low = ΧΑΜΗΛΑ 500 FT
planner-medium = ΜΕΣΑΙΑ 5000 FT
planner-high = ΥΨΗΛΑ 20000 FT
planner-title = ΣΧΕΔΙΑΣΤΗΣ ΑΠΟΣΤΟΛΗΣ
planner-route = Διαδρομή
planner-base = ΒΑΣΗ
planner-total = ΣΥΝΟΛΟ { $km } KM  { $time }
planner-briefed = Διαδρομή briefing { $km } km, καύσιμο { $fuel } lbs
planner-coverage = Κάλυψη radar για
    .note = Η κάλυψη radar προϋποθέτει οριζόντια πτήση στα 480 kt με κλειστές τις θύρες του καμπαρτίνου.
planner-detected = { $km } km της διαδρομής εντοπίστηκαν
planner-pulse = Εμβέλεια ανίχνευσης radar παλμών (P)
planner-doppler = Εμβέλεια ανίχνευσης radar Doppler (D)
planner-missile = Εμβέλεια πυραύλων
planner-uav = Εχθρικό UAV
planner-edit-hint = Σύρετε ένα σημείο για να το μετακινήσετε, τον χάρτη για μετατόπιση. Κλικ σε τμήμα για νέο σημείο, δεξί κλικ για αφαίρεση. Ροδάκι: ζουμ.
planner-reset = ΕΠΑΝΑΦΟΡΑ
planner-view-hint = Σύρετε τον χάρτη για μετατόπιση. Ροδάκι: ζουμ.

## Αποθηκευμένες αποστολές

saves-title = ΑΠΟΘΗΚΕΥΜΕΝΕΣ ΑΠΟΣΤΟΛΕΣ
saves-subtitle = Αποθηκεύονται εν πτήσει από το μενού: ΑΠΟΘΗΚΕΥΣΗ ΑΠΟΣΤΟΛΗΣ
saves-none = Καμία αποθηκευμένη αποστολή
saves-line = { $theater }  -  { $aircraft }  -  χρόνος αποστολής { $time }
saves-saved-at = Αποθηκεύτηκε { $date }
saves-unreadable = Μη αναγνώσιμη αποθήκευση (άλλη έκδοση)
saves-delete =
    .facts = Διαγραφή αυτής της αποθήκευσης
saves-back =
    .facts = Επιστροφή στην οθόνη μεμονωμένης αποστολής
