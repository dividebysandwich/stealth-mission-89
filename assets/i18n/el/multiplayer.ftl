# Το παράθυρο της εκστρατείας (src/campaign/menu.rs), το παράθυρο πολλών παικτών
# (src/multiplayer_menu.rs), το υπόμνημα και η ζωντανή προβολή του χάρτη πολέμου
# (src/campaign/picture.rs, unit_view.rs).

delete = ΔΙΑΓΡΑΦΗ
delete-campaign = ΔΙΑΓΡΑΦΗ ΑΥΤΗΣ ΤΗΣ ΕΚΣΤΡΑΤΕΙΑΣ;
delete-save = ΔΙΑΓΡΑΦΗ ΑΥΤΗΣ ΤΗΣ ΑΠΟΘΗΚΕΥΣΗΣ;
delete-permanent = Η διαγραφή είναι οριστική

## Παράθυρο εκστρατείας

cm-group-campaign = Εκστρατεία
cm-weather-dynamic = Δυναμικός
    .facts =
        Ο καιρός αλλάζει κάθε 10-22 ώρες του ρολογιού του ουρανού
        Τα σύννεφα εμποδίζουν την οπτική επαφή και για τις δύο πλευρές
cm-weather-clear = Αίθριος ουρανός
    .facts = Χωρίς σύννεφα σε όλη την εκστρατεία
cm-section-day-night = Ημέρα και νύχτα
cm-day-real = Πραγματικός χρόνος
    .facts = Μια ημέρα διαρκεί 24 ώρες
cm-day-2h = 2 ώρες / ημέρα
    .facts = Μια ημέρα διαρκεί 2 ώρες
cm-day-1h = 1 ώρα / ημέρα
    .facts = Μια ημέρα διαρκεί 1 ώρα
cm-day-30min = 30 λεπτά / ημέρα
    .facts = Μια ημέρα διαρκεί 30 λεπτά
cm-conditions-note = Οι συνθήκες ισχύουν για μια νέα εκστρατεία
cm-rules-note = Μια αποθηκευμένη εκστρατεία διατηρεί τους κανόνες με τους οποίους ξεκίνησε
cm-about =
    Ο πόλεμος εξελίσσεται σε πραγματικό χρόνο όσο παίζετε, ακόμη και με ανοιχτό το μενού.
    Πετάξτε τις αποστολές του πίνακα: αναγνώριση, SEAD, CAP, CAS, anti-ship, strike, συνοδεία.
    Οι αποβάσεις με ελικόπτερα καταλαμβάνουν τις πόλεις και τα αεροδρόμια του εχθρού.
    Δεν σκοτώνεστε και δεν αιχμαλωτίζεστε ποτέ: μετά από μια εξόρμηση επιστρέφετε στον πίνακα.
cm-section-new = Νέα εκστρατεία
cm-new-line = { $theater }  -  { $situation }  -  αντίπαλοι { $opponents }
cm-start = ΕΝΑΡΞΗ
cm-section-continue = Συνέχεια
cm-no-saves = Καμία αποθηκευμένη εκστρατεία
cm-played = { $hours } ώρες παιχνιδιού
cm-save-gone = Η αποθήκευση δεν υπάρχει πια
cm-save-unreadable = Η αποθήκευση δεν μπόρεσε να διαβαστεί: { $error }
cm-save-undeletable = Η αποθήκευση δεν μπόρεσε να διαγραφεί: { $error }
cm-in-vr = ΣΕ VR
cm-delete =
    .facts = Διαγραφή αυτής της αποθηκευμένης εκστρατείας

## Χάρτης πολέμου

pic-friendly-aircraft = Φίλιο αεροσκάφος
pic-friendly-helicopter = Φίλιο ελικόπτερο
pic-enemy-fighter = Εχθρικό μαχητικό (στο radar)
pic-sam-known = Θέση SAM (γνωστή)
pic-guns-known = Πυροβόλα / MANPADS (γνωστά)
pic-reconned = Αναγνωρισμένο
pic-you = Εσείς
pic-site = Θέση: { $place }
uv-live = { $unit } - ΖΩΝΤΑΝΑ
uv-contact-lost = ΑΠΩΛΕΙΑ ΕΠΑΦΗΣ
uv-hint = Σύρσιμο: περιστροφή  Ροδάκι: ζουμ

## Πολλοί παίκτες

mp-disconnected = Αποσυνδέθηκε: { $why }
mp-connecting = Σύνδεση με { $address }...
mp-joining = Συνδέθηκε με { $address }, είσοδος...
mp-browser-public-only = από τον browser μπορείτε να συνδεθείτε μόνο σε servers της δημόσιας λίστας
mp-title-host = ΠΟΛΛΟΙ ΠΑΙΚΤΕΣ: ΔΗΜΙΟΥΡΓΙΑ ΠΑΙΧΝΙΔΙΟΥ
mp-subtitle = Συνεργασία στη δυναμική εκστρατεία: πετάξτε μαζί στον πόλεμο
mp-your-name = Το όνομά σας
    .hint = Όνομα πιλότου
mp-join-address = Σύνδεση με διεύθυνση
    .hint = διεύθυνση ή όνομα[:θύρα]
mp-connect = ΣΥΝΔΕΣΗ
mp-server-name = Όνομα server
mp-udp-port = Θύρα UDP
mp-address-note = Οι άλλοι παίκτες συνδέονται με τη διεύθυνση αυτού του υπολογιστή· ανοίξτε τη θύρα UDP στον router σας για να παίξετε μέσω διαδικτύου
mp-list-public = Εμφάνιση στη δημόσια λίστα servers (οι άλλοι βρίσκουν το παιχνίδι στη λίστα τους)
mp-rules = Κανόνες για όλους
mp-time-of-day = Ώρα της ημέρας (νύχτες με φεγγάρι)
mp-realistic-weapons = Ρεαλιστικά όπλα F-117A
mp-friendly-fire = Φίλια πυρά
mp-day-night = Ημέρα και νύχτα:
mp-weather = Καιρός:
mp-host-new = ΝΕΟ ΠΑΙΧΝΙΔΙ
mp-host-saved = Φιλοξενία αποθηκευμένης εκστρατείας
mp-host = ΦΙΛΟΞΕΝΙΑ
mp-host-game = ΔΗΜΙΟΥΡΓΙΑ ΠΑΙΧΝΙΔΙΟΥ
mp-port-error = Θύρα UDP { $port }: { $error }
mp-default-server = Εκστρατεία του/της { $pilot }
mp-public-servers = Δημόσιοι servers
mp-refresh = ΑΝΑΝΕΩΣΗ
mp-server-count = { $servers } servers, { $pilots } πιλότοι
mp-list-error = Η λίστα των servers δεν είναι προσβάσιμη: { $error }
mp-no-servers = Κανένας δημόσιος server αυτή τη στιγμή
mp-col-server = SERVER
mp-col-pilots = ΠΙΛΟΤΟΙ
mp-col-war = ΠΟΛΕΜΟΣ
mp-dedicated = DEDICATED
mp-other-version = άλλη έκδοση
mp-join = ΕΙΣΟΔΟΣ
