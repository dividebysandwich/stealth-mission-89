# Το παράθυρο ΕΠΙΛΟΓΕΣ (src/options.rs) και οι σελίδες του στο μενού πτήσης.

options-title = ΕΠΙΛΟΓΕΣ
options-tab-game = ΠΑΙΧΝΙΔΙ
options-tab-graphics = ΓΡΑΦΙΚΑ
options-tab-sound = ΗΧΟΣ
options-tab-controls = ΧΕΙΡΙΣΜΟΙ
options-footer = Αποθηκεύονται για τις επόμενες πτήσεις· οι διακόπτες ρεαλισμού επιλέγονται σε κάθε παιχνίδι

## ΠΑΙΧΝΙΔΙ

options-section-language = Γλώσσα
options-section-classic-modern = Κλασικό / Μοντέρνο
options-cockpit = Πιλοτήριο
    .facts =
        Μοντέρνο: οθόνες υψηλής ανάλυσης με κουμπιά στο πλαίσιο
        Κλασικό: η διάταξη του πίνακα οργάνων της εποχής DOS και οθόνες 77x60 (F-117A)
options-hud = HUD
    .facts =
        Μοντέρνο: πράσινο HUD υψηλής ανάλυσης τύπου F-16 με λειτουργίες NAV, AA και AG
        Κλασικό: το HUD της εποχής DOS
options-avionics = Ηλεκτρονικά πτήσης
    .facts =
        Μοντέρνο: φορμά MFD τύπου F-16 σε κάθε MFD (οθόνη κατάστασης, HARM, targeting pod, χάρτης, φορτία) με κέρσορα που μετακινείται
        Κλασικό: χάρτης και TAC αριστερά, κάμερα, WAY, WPN και ORD δεξιά
options-flight-model = Μοντέλο πτήσης
    .facts =
        Μοντέρνο: αεροδυναμικό μοντέλο πτήσης με fly-by-wire: το stick ελέγχει τον ρυθμό πρόνευσης και κλίσης· αν το αφήσετε, η μύτη μένει στη θέση της και η τροχιά διατηρείται
        Κλασικό: μοντέλο πτήσης της εποχής DOS
options-section-flying = Πτήση
options-auto-trim = Auto trim
    .facts =
        Με το stick ελεύθερο, το trim κρατά τη στάση όπου την αφήσατε
        Ελικόπτερα: η μύτη δεν ανεβαίνει πια με την ταχύτητα· τζετ: εξισορροπεί τις κατεστραμμένες επιφάνειες ελέγχου
        Χειροκίνητο trim: Alt + βελάκια, το Alt + Home το κεντράρει
options-tooltips = Tooltips πιλοτηρίου
    .facts = Τι είναι ένα χειριστήριο ή μια οθόνη του πιλοτηρίου, κάτω από το ποντίκι ή την άκρη του δαχτύλου σε VR
options-section-vr-size = Μέγεθος πιλοτηρίου σε VR
options-vr-size =
    .facts =
        Μέγεθος του πιλοτηρίου (και του κόσμου) στο headset
        100%: φυσικό μέγεθος

## ΓΡΑΦΙΚΑ

options-section-rendering = Απόδοση εικόνας
options-shaders = Shaders
    .facts =
        Μοντέρνο: διακριτική κατευθυντική σκίαση, ομαλή ομίχλη αποστάσεων
        Κλασικό: επίπεδη σκίαση
options-shadows = Σκιές
    .facts =
        Μοντέρνο: σκιές του ήλιου στο πιλοτήριο και στον κόσμο (κοστίζει GPU)
        Κλασικό: χωρίς σκιές
options-point-lights = Σημειακά φώτα
    .facts =
        Μοντέρνο: φωτισμός πιλοτηρίου και οθονών· οι εκρήξεις, οι καπνοί και τα φώτα διαδρόμου φωτίζουν τη σκηνή
        Κλασικό: μόνο φως ήλιου και ουρανού
options-time-of-day = Ώρα της ημέρας
    .facts =
        Μοντέρνο: ο ήλιος και η σελήνη κινούνται με το ρολόι της αποστολής, ο ουρανός σκοτεινιάζει και φωτίζει με τον ήλιο· αστέρια και φάση της σελήνης τη νύχτα
        Μοντέρνο: το φεγγαρόφωτο ορίζει από πόσο μακριά σας βλέπουν οι εχθρικοί πιλότοι τη νύχτα (2 km σε νύχτα χωρίς σελήνη, 4 km με πανσέληνο)
        Κλασικό: αργή αλλαγή χρωμάτων και σταθερός ήλιος
options-msaa = Anti-aliasing (MSAA)
    .facts =
        Ομαλές ακμές: 4x MSAA, 2x σε VR (κοστίζει GPU)
        Απενεργοποιημένο: σκληρές ακμές pixel
options-auto-detail = Αυτόματη λεπτομέρεια τοπίου (VR)
    .facts =
        Στο headset, η εμβέλεια των πόλεων, των δασών και των χωραφιών μικραίνει όταν τα καρέ καθυστερούν και μεγαλώνει ξανά όταν υπάρχει περιθώριο
        Οι ρυθμίσεις γραφικών μένουν όπως τις επιλέξατε
options-sun-moon = Ήλιος και σελήνη
    .facts =
        Ο ήλιος στον ουρανό την ημέρα, η σελήνη και τα αστέρια τη νύχτα
        Η σελήνη δείχνει τη φάση της αποστολής

## ΗΧΟΣ

options-section-volume = Ένταση
options-volume-master = Γενική
    .facts = Όλοι οι ήχοι
options-volume-engine = Κινητήρας
    .facts = Ο κινητήρας ή ο ρότορας σας, και τα αεροσκάφη που ακούγονται γύρω σας
options-volume-effects = Εφέ
    .facts = Όπλα, εκρήξεις, σύστημα προσγείωσης, ασύρματος και τα υπόλοιπα
options-volume-warnings = Προειδοποιήσεις
    .facts = Σειρήνα απώλειας στήριξης, προειδοποιήσεις radar και πυραύλων, σειρήνα φωτιάς, τόνοι Sidewinder
options-section-cockpit = Πιλοτήριο
options-low-pass = Φίλτρο χαμηλών κινητήρα
    .facts =
        Μέσα στο αεροσκάφος ο κινητήρας ακούγεται πνιχτά (το κάλυμμα και το κράνος κόβουν τις υψηλές συχνότητες)
        Απενεργοποιημένο: ίδιος ήχος με το εξωτερικό

## ΧΕΙΡΙΣΜΟΙ

options-section-bindings = Αντιστοιχίσεις
options-keys-joystick = ΠΛΗΚΤΡΑ ΚΑΙ JOYSTICK
    .facts =
        Πλήκτρα χάρτη, άξονες και κουμπιά joystick και HOTAS
        Αποθηκεύονται για τις επόμενες πτήσεις
options-section-head-tracking = Παρακολούθηση κεφαλιού/ματιών
options-head-tracking = Παρακολούθηση κεφαλιού/ματιών
    .facts =
        Το κεφάλι σας γυρίζει και μετακινεί την εικόνα στο πιλοτήριο (σε υπολογιστή· σε VR το κάνει το headset)
        Πρόγραμμα παρακολούθησης (OpenTrack...): έξοδος FreeTrack ή UDP στο 127.0.0.1, θύρα 4242
        Home: η τρέχουσα θέση του κεφαλιού γίνεται το κέντρο
options-tracking = Παρακολούθηση ({ $source })
options-no-tracker = Δεν ακούστηκε κανένας αισθητήρας: ξεκινήστε το πρόγραμμα παρακολούθησης

## EFFECTS

options-section-effects = Effects
options-section-retro = Retro
effect-shake = Camera shake
    .facts =
        The view shakes with the gun, nearby blasts, stall buffet, touchdowns and the runway
        VR: the cockpit vibrates, your head stays still
effect-flashes = Combat flashes
    .facts =
        Flak bursts, explosions and missile launches light the clouds, the ground and the cockpit for a moment
effect-barrage = Night barrage
    .facts =
        Conventional war at night: the towns' guns fire tracer blindly into the sky, searchlights sweep over them
        Only for the look: the barrage hits nothing, the searchlights find nothing
effect-cockpit-night = Cockpit night lighting
    .facts =
        A dim red flood light in the cockpit after dark (blue-green under the goggles)
        The master caution pulses amber over the panel while a warning is on
effect-nvg = Night vision goggles
    .facts =
        Bright lights bloom into halos, the picture grains more in the dark
        Desktop: the goggles' round field of view
effect-snow = Snow
    .facts =
        Above the freezing level storms snow instead of rain: low in a northern winter, high in summer
effect-mist = Ground mist
    .facts =
        Modern shaders: mist lies over the low ground and the sea at dawn, dusk and night
        Thin in the deserts, none in storms
effect-cloud-shadows = Cloud shadows
    .facts =
        Modern shaders: the clouds' shadows on the ground below
effect-vapour = Vapour and downwash
    .facts =
        Pulling hard, vapour trails from the wingtips
        A helicopter hovering low whips up dust, or spray over water
effect-night-sky = Night sky
    .facts =
        Shooting stars now and then, the stars low over the horizon twinkle
        The northern lights over the North Cape
effect-vga = VGA colours
    .facts =
        The 1989 look with modern shading: 64 steps a colour channel, as a VGA card's, dithered
effect-scanlines = Scanlines
    .facts =
        A CRT monitor's scanlines (not in VR)

## LANGUAGE NAMES

lang-name-en = Αγγλικά
lang-name-bg = Βουλγαρικά
lang-name-zh-Hans = Κινεζικά (απλοποιημένα)
lang-name-hr = Κροατικά
lang-name-cs = Τσεχικά
lang-name-da = Δανικά
lang-name-nl = Ολλανδικά
lang-name-et = Εσθονικά
lang-name-fi = Φινλανδικά
lang-name-fr = Γαλλικά
lang-name-de = Γερμανικά
lang-name-el = Ελληνικά
lang-name-hu = Ουγγρικά
lang-name-id = Ινδονησιακά
lang-name-ga = Ιρλανδικά
lang-name-it = Ιταλικά
lang-name-ja = Ιαπωνικά
lang-name-ko = Κορεατικά
lang-name-lv = Λετονικά
lang-name-lt = Λιθουανικά
lang-name-mt = Μαλτέζικα
lang-name-pl = Πολωνικά
lang-name-pt-BR = Πορτογαλικά (Βραζιλίας)
lang-name-ro = Ρουμανικά
lang-name-ru = Ρωσικά
lang-name-sk = Σλοβακικά
lang-name-sl = Σλοβενικά
lang-name-es = Ισπανικά
lang-name-sv = Σουηδικά
lang-name-uk = Ουκρανικά

## Browser build: offer to play in the browser's language (src/i18n.rs)

language-offer-question = Γλώσσα του προγράμματος περιήγησης: { $language }. Θέλετε να παίξετε σε αυτή τη γλώσσα;
language-offer-yes = Ναι
language-offer-no = Όχι
language-offer-never = Να μην εμφανιστεί ξανά
effect-sunbeams = Sunbeams
    .facts =
        Shafts of sunlight through the gaps of a broken overcast
