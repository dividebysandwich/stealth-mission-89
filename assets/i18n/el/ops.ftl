# Η οθόνη επιχειρήσεων της εκστρατείας (src/campaign/ops.rs).

ops-thunderstorms = ΚΑΤΑΙΓΙΔΕΣ
ops-wx-short = WX { $now }, { $next } ΣΤΙΣ { $at }
ops-wx-line = { $weather } Αλλάζει σε { $next } στις { $at }.
ops-clock = ΗΜΕΡΑ { $day }  { $time }
ops-landed-at = ΠΡΟΣΓΕΙΩΣΗ ΣΤΗ ΒΑΣΗ { $base }
ops-ground-crew = Το πλήρωμα εδάφους είναι έτοιμο
ops-rearm = ΕΠΑΝΟΠΛΙΣΜΟΣ & ΑΝΕΦΟΔΙΑΣΜΟΣ
ops-end-sortie = ΤΕΡΜΑΤΙΣΜΟΣ ΕΞΟΡΜΗΣΗΣ
ops-on-server = ΣΤΟΝ { $server }
ops-serving = ΦΙΛΟΞΕΝΙΑ
ops-tab-missions = ΑΠΟΣΤΟΛΕΣ
ops-tab-map = ΧΑΡΤΗΣ ΠΟΛΕΜΟΥ
ops-tab-air-defence = ΑΕΡΑΜΥΝΑ
ops-tab-forces = ΔΥΝΑΜΕΙΣ
ops-tab-players = ΠΑΙΚΤΕΣ
ops-tab-debrief = DEBRIEF
ops-tab-log = ΗΜΕΡΟΛΟΓΙΟ
ops-tab-chat = CHAT
ops-blue = ΜΠΛΕ { $count }
ops-red = ΚΟΚΚΙΝΟΙ { $count }
ops-war-over = Ο πόλεμος τελείωσε
ops-running = Η προσομοίωση του πολέμου εκτελείται
ops-save-exit = ΑΠΟΘΗΚΕΥΣΗ & ΕΞΟΔΟΣ
ops-disconnect = ΑΠΟΣΥΝΔΕΣΗ
ops-leave-local = ΑΠΟΘΗΚΕΥΣΗ ΚΑΙ ΕΞΟΔΟΣ ΑΠΟ ΤΗΝ ΕΚΣΤΡΑΤΕΙΑ;
    .text = Ο πόλεμος σταματά μέχρι να τον συνεχίσετε. Τα αεροσκάφη στον αέρα χάνονται· οι αποβάσεις που βρίσκονται σε εξέλιξη ξεκινούν ξανά από τη βάση τους.
ops-leave-host = ΑΠΟΘΗΚΕΥΣΗ ΚΑΙ ΔΙΑΚΟΠΗ ΤΗΣ ΦΙΛΟΞΕΝΙΑΣ ΤΗΣ ΕΚΣΤΡΑΤΕΙΑΣ;
    .text = Οι παίκτες που συνδέθηκαν αποσυνδέονται και ο πόλεμος σταματά μέχρι να τον συνεχίσετε. Τα αεροσκάφη στον αέρα χάνονται· οι αποβάσεις που βρίσκονται σε εξέλιξη ξεκινούν ξανά από τη βάση τους.
ops-leave-client = ΑΠΟΣΥΝΔΕΣΗ ΑΠΟ ΤΟΝ SERVER;
    .text = Η εκστρατεία συνεχίζεται στον server· τίποτα δεν αποθηκεύεται εδώ. Η εξόρμησή σας τελειώνει.
ops-unit-destroyed = Η μονάδα { $unit } καταστράφηκε: επιστροφή στις επιχειρήσεις

## Αποστολές

ops-section-missions = Αποστολές
ops-threat = ΑΠΕΙΛΗ { $threat }
ops-no-missions = Καμία αποστολή: το μέτωπο είναι ήσυχο
ops-section-underway = Αποβάσεις και επιδρομές
ops-enemy-liftoff = ΕΧΘΡΟΣ σε { $place }: απογείωση σε { $mins } λεπτά
ops-liftoff = { $place }: απογείωση σε { $mins } λεπτά
ops-enemy-helis = ΕΧΘΡΟΣ σε { $place }: { $count } ελικόπτερα πλησιάζουν
ops-helis = { $place }: { $count } ελικόπτερα καθ' οδόν
ops-raid-sead = SEAD της AI σε { $place }
ops-raid-drones = DRONES σε { $place }
ops-raid = ΕΠΙΔΡΟΜΗ στο αεροδρόμιο { $place }
ops-raid-launch = { $raid }: εκκίνηση σε { $mins } λεπτά
ops-raid-on-way = { $raid }: { $count } καθ' οδόν
ops-resupply-ai = ΑΝΕΦΟΔΙΑΣΜΟΣ της μονάδας { $unit }: UH-60 καθ' οδόν
ops-awacs = ΕΧΘΡΙΚΟ AWACS στον αέρα για άλλα { $mins } λεπτά
ops-down = ΚΑΤΩ: { $pilot } - διάσωση μέσα σε { $mins } λεπτά
ops-aboard = ΕΠΙΒΙΒΑΣΤΗΚΕ: { $pilot } - καθ' οδόν προς το σπίτι
ops-none-underway = Καμία σε εξέλιξη
ops-ai-helicopter = Ένα ελικόπτερο της AI την αναλαμβάνει σε { $time } εκτός αν την αναλάβει πιλότος UH-60
ops-defences = Άμυνες: { $sams } θέσεις SAM, απειλή { $threat }
ops-recon-intel = ΠΛΗΡΟΦΟΡΙΕΣ ΑΝΑΓΝΩΡΙΣΗΣ
ops-order-insertion = ΔΙΑΤΑΓΗ ΑΠΟΒΑΣΗΣ
    .facts = Πετάξτε τη συνοδεία για να τα καλύψετε ή οδηγήστε τα με το UH-60
ops-from = Από
ops-planned-by-group = Σχεδιάστηκε από την ομάδα
ops-stores = { $aircraft } - φορτία: { $stores }
ops-fly-uh60 = ΠΕΤΑΞΤΕ ΤΟ UH-60
ops-accept = ΑΠΟΔΟΧΗ ΑΠΟΣΤΟΛΗΣ
ops-planning = ΣΧΕΔΙΑΣΜΟΣ: { $pilots }
ops-flying = ΣΕ ΠΤΗΣΗ: { $pilots }
ops-package = PACKAGE με { $kinds }: επιτύχετε δύο είδη εδώ μέσα σε { $mins } λεπτά, +{ $bonus } πόντοι το καθένα
ops-sead-on-way = Το SEAD της AI είναι καθ' οδόν
ops-request-sead = ΑΙΤΗΣΗ SEAD ΤΗΣ AI ({ $cost } υλικό, { $aircraft } αεροσκάφη)
    .facts = Φίλια αεροσκάφη κρούσης πηγαίνουν εναντίον μιας θέσης SAM αυτού του τόπου πριν από εσάς
    .disabled = Δεν υπάρχει αρκετό υλικό ή αεροσκάφη στο έδαφος (δείτε ΔΥΝΑΜΕΙΣ)

## Κανόνες, υλικό, δυνάμεις

ops-rule-damage = ρεαλιστικές ζημιές
ops-rule-guidance = ρεαλιστική καθοδήγηση
ops-rule-no-crashes = χωρίς συντριβές
ops-rule-time = συνεχής χρόνος
ops-rule-weapons = ρεαλιστικά όπλα F-117A
ops-rule-friendly-fire = φίλια πυρά
ops-rule-classic = κλασικό
ops-materiel = ΥΛΙΚΟ
ops-materiel-blue = ΜΠΛΕ { $materiel } (+{ $income }/ώρα)
ops-materiel-red = ΚΟΚΚΙΝΟΙ { $materiel } (+{ $income }/ώρα)
ops-friendly = ΦΙΛΙΟΙ
ops-enemy = ΕΧΘΡΟΣ
ops-section-forces = Δυνάμεις
ops-places-held = Τόποι υπό έλεγχο
ops-materiel-row = Υλικό
ops-income = Εισόδημα ανά ώρα
ops-income-from = από
ops-income-sources = { $base } + { $places } τόποι x { $per_place } + { $cities } πόλεις x { $per_city }
ops-fighters-ground = Μαχητικά στο έδαφος
ops-helicopters-ground = Ελικόπτερα στο έδαφος
ops-in-air = Στον αέρα
ops-in-air-value = { $fighters } μαχητικά, { $strike } κρούσης, { $helicopters } ελικόπτερα
ops-since-start = Από την αρχή
ops-earned = κέρδη
ops-repairs = επισκευές
ops-operations = εξορμήσεις, επιδρομές, αποβάσεις
ops-new-aircraft = νέα αεροσκάφη
ops-lost = απώλειες (πόλεις, πιλότοι)
ops-section-how = Πώς λειτουργεί
ops-how =
    Κάθε πλευρά κερδίζει υλικό κάθε ώρα: ένα βασικό ποσό, περισσότερο για κάθε τόπο που κατέχει και για κάθε άθικτη πόλη ανάμεσά τους. Βομβαρδίστε τις πόλεις του εχθρού (INTERDICTION) για να περικόψετε το εισόδημά του.
    Η AI πληρώνει για όλα όσα κάνει: επισκευές, κάθε εξόρμηση μαχητικών, επιδρομές και τις δικές της αποβάσεις. Αν της λείψει υλικό, οι ζημιές της μένουν ανεπισκεύαστες και τα αεροσκάφη της μένουν στη βάση.
    Η AI πετά μόνο τα αεροσκάφη που έχει· όσα καταρριφθούν χάνονται, όσα προσγειωθούν πετούν ξανά. Κάθε ώρα κάθε πλευρά αγοράζει νέα όσο διαρκεί το υλικό της.
    Σε εσάς δεν λείπει ποτέ τίποτα: τα αεροσκάφη σας, οι βάσεις σας (βομβαρδισμένες ή όχι) και τα ελικόπτερα των δικών σας διαταγών απόβασης είναι πάντα διαθέσιμα.
ops-section-prices = Τιμές
ops-price-airfield = Επισκευή αεροδρομίου
ops-price-sam = Επισκευή θέσης SAM
ops-price-city = Ανοικοδόμηση πόλης
ops-price-ship = Επισκευή πλοίου
ops-price-gun = Πυροβόλο ή ομάδα MANPADS
ops-price-sortie = Εξόρμηση μαχητικού
ops-price-raid = Επιδρομή
ops-price-insertion = Απόβαση της AI
ops-price-fighter = Νέο μαχητικό
ops-price-helicopter = Νέο ελικόπτερο
ops-price-city-lost = Πόλη καταστράφηκε (απώλεια)
ops-price-captured = Πιλότος αιχμαλωτίστηκε (απώλεια)

## Χάρτης πολέμου

ops-click-to-watch = { $unit } - κλικ για παρακολούθηση
ops-section-legend = Υπόμνημα
ops-legend-friendly-airfield = Φίλιο αεροδρόμιο
ops-legend-enemy-airfield = Εχθρικό αεροδρόμιο
ops-legend-city = Φίλια / εχθρική πόλη
ops-legend-airfield = Φίλιο / εχθρικό αεροδρόμιο
ops-legend-insertion = Διαδρομή απόβασης
ops-watch-hint = Κάντε κλικ σε μια μονάδα για ζωντανή παρακολούθηση
ops-map-hint = Ροδάκι: ζουμ, σύρσιμο: μετατόπιση
ops-patriot = Πυροβολαρχία Patriot
ops-drone-center = Κέντρο διοίκησης drones
ops-manned-by = Επανδρώνεται από: { $pilot }
ops-manned-by-other = Την επανδρώνει άλλος παίκτης
ops-control = ΕΛΕΓΧΟΣ
    .facts = Αναλάβετε την κονσόλα της

## Debrief, τέλος, ημερολόγιο, chat

ops-no-sortie = Καμία εξόρμηση ακόμη
ops-section-sortie = Εξόρμηση
ops-achieved = Ο ΣΤΟΧΟΣ ΕΠΙΤΕΥΧΘΗΚΕ
ops-not-achieved = Ο ΣΤΟΧΟΣ ΔΕΝ ΕΠΙΤΕΥΧΘΗΚΕ
ops-waiting-raid = ΑΝΑΜΟΝΗ ΤΗΣ ΕΠΙΔΡΟΜΗΣ
ops-waiting-insertion = ΑΝΑΜΟΝΗ ΤΗΣ ΑΠΟΒΑΣΗΣ
ops-flight-time = Χρόνος πτήσης { $mins } λεπτά
ops-kills = Καταρρίψεις: { $air } αεροσκάφη, { $ground } στο έδαφος
ops-points = { $points } πόντοι
ops-points-bonus = { $points } πόντοι (μπόνους package +{ $bonus })
ops-campaign-points = { $rank } - { $points } πόντοι εκστρατείας
ops-campaign-record = Εκστρατεία: { $sorties } εξορμήσεις, { $won } στόχοι επιτεύχθηκαν
ops-section-meanwhile = Στο μεταξύ
ops-victory = ΝΙΚΗ
    .text = Ο εχθρός δεν κατέχει πια κανένα αεροδρόμιο: το θέατρο επιχειρήσεων είναι δικό μας.
ops-defeat = ΗΤΤΑ
    .text = Δεν απέμεινε κανένα φίλιο αεροδρόμιο: το θέατρο επιχειρήσεων χάθηκε.
ops-end-day = Ημέρα { $day }. Φίλιοι τόποι { $friendly }, εχθρικοί τόποι { $enemy }.
ops-next-campaign = Η ΕΠΟΜΕΝΗ ΕΚΣΤΡΑΤΕΙΑ ΞΕΚΙΝΑ ΣΕ { $time }
ops-next-campaign-now = Η ΕΠΟΜΕΝΗ ΕΚΣΤΡΑΤΕΙΑ ΞΕΚΙΝΑ
ops-next-from-menu = Ξεκινήστε την επόμενη εκστρατεία από το κύριο μενού.
ops-section-log = Ημερολόγιο εκστρατείας
ops-section-chat = Chat
ops-no-messages = Κανένα μήνυμα ακόμη
ops-message = Μήνυμα
    .hint = Πληκτρολογήστε ένα μήνυμα, το Enter το στέλνει
ops-send = ΑΠΟΣΤΟΛΗ

## Briefing

ops-briefing = BRIEFING ΑΠΟΣΤΟΛΗΣ
ops-briefing-line = Από { $base } - { $km } km ως τον στόχο - { $aircraft } - φορτία: { $stores }
ops-route-locked = Η διαδρομή είναι κλειδωμένη: πετά ο/η { $pilots }
ops-route-threats = ΔΙΑΔΡΟΜΗ & ΑΠΕΙΛΕΣ
ops-take-off = ΑΠΟΓΕΙΩΣΗ
ops-radars-shown = Radar που εμφανίζονται: εντοπίστηκαν από φίλια αεροσκάφη και λειτουργούν ακόμη

## Παίκτες

ops-section-players = Παίκτες
ops-col-name = ΟΝΟΜΑ
ops-col-status = ΚΑΤΑΣΤΑΣΗ
ops-col-sorties = ΕΞΟΡΜΗΣΕΙΣ
ops-col-points = ΠΟΝΤΟΙ
ops-col-pilot = ΠΙΛΟΤΟΣ
ops-col-objectives = ΣΤΟΧΟΙ
ops-col-kills = ΚΑΤΑΡΡΙΨΕΙΣ ΑΕΡΑ/ΕΔΑΦΟΣ
ops-col-medal = ΚΑΛΥΤΕΡΟ ΜΕΤΑΛΛΙΟ
ops-you = (εσείς)
ops-host = ΟΙΚΟΔΕΣΠΟΤΗΣ
ops-status-planning = Σχεδιάζει: { $title }
ops-status-board = Στον πίνακα αποστολών
ops-status-flying = Πετά: { $title } - { $aircraft }
ops-status-reconnecting = Η σύνδεση χάθηκε - το αεροσκάφος κρατείται { $seconds } s
ops-flown = { $flown } ({ $won } επιτεύχθηκαν)
ops-section-leaderboard = Κατάταξη
ops-alone = Πετάτε μόνοι. Για να πετάξετε μαζί με άλλους, φιλοξενήστε αυτή την εκστρατεία από το ΠΟΛΛΟΙ ΠΑΙΚΤΕΣ στο κύριο μενού.
ops-rules = Κανόνες αυτής της εκστρατείας (διατηρούνται από την αρχή της): { $rules }
ops-no-sorties = Καμία εξόρμηση ακόμη
