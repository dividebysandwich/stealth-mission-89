# Το debrief της αποστολής (src/debrief.rs) και η αναφορά της προσομοίωσης (stealth-sim score.rs).

## Πώς τελείωσε μια πτήση

outcome-landed = Ασφαλής προσγείωση
outcome-bad-landing = Κακή προσγείωση
outcome-ejected = Εκτίναξη - προσγείωση με αλεξίπτωτο
outcome-rescued = Εκτίναξη - διάσωση από CSAR
outcome-captured = Εκτίναξη - αιχμαλωσία
outcome-crashed = Συντριβή
outcome-ejection-failed = Αποτυχία εκτίναξης
outcome-aborted = Η αποστολή διακόπηκε
outcome-collision = Σύγκρουση

## Βαθμολογίες

rating-unsatisfactory = Ανεπαρκής
rating-marginal = Οριακή
rating-satisfactory = Ικανοποιητική
rating-excellent = Εξαιρετική
rating-outstanding = Άριστη

## Πόντοι

points-primary = Κύριος στόχος
points-secondary = Δευτερεύων στόχος
points-aircraft = Αεροσκάφη που καταστράφηκαν
points-unseen = Καταρρίψεις χωρίς να σας δουν
points-ground = Στόχοι εδάφους που καταστράφηκαν
points-drones = Drones που καταρρίφθηκαν
points-drones-through = Drones που πέρασαν
points-nuclear = Πλήγμα στον πυρηνικό σταθμό
points-detected = Εντοπισμός
points-missile-hits = Πλήγματα από πυραύλους

## Χρονολόγιο (τα ονόματα τόπων και αεροσκαφών όπως στην αποστολή)

timeline-primary = {" "}(κύριος)
timeline-secondary = {" "}(δευτερεύων)
timeline-destroyed = Καταστροφή: { $place }{ $tag }
timeline-destroyed-ground = Καταστροφή στόχου εδάφους{ $tag }
timeline-shot-down = Κατάρριψη: { $aircraft }{ $tag }
timeline-hit = Πλήγμα από πύραυλο
timeline-seen = Εντοπισμός από: { $aircraft }
timeline-radar-id = Ταυτοποίηση radar από: { $place }
timeline-objective = Ο στόχος επιτεύχθηκε{ $tag }
timeline-got-through = Πέρασε: { $aircraft }
timeline-photo = Λήψη φωτογραφίας{ $tag }

## Το παράθυρο του debrief

debrief-title = DEBRIEF ΑΠΟΣΤΟΛΗΣ
debrief-training = ΑΠΟΣΤΟΛΗ ΕΚΠΑΙΔΕΥΣΗΣ
debrief-results = ΑΠΟΤΕΛΕΣΜΑΤΑ
debrief-flight-map = ΧΑΡΤΗΣ ΠΤΗΣΗΣ
debrief-section-results = Αποτελέσματα
debrief-accomplished = ΕΠΙΤΕΥΧΘΗΚΕ
debrief-failed = ΑΠΟΤΥΧΙΑ
debrief-primary = ΚΥΡΙΟΣ
debrief-secondary = ΔΕΥΤΕΡΕΥΩΝ
debrief-aircraft = ΑΕΡΟΣΚΑΦΗ
debrief-drones = DRONES
debrief-drones-value = { $down } καταρρίφθηκαν, { $through } πέρασαν
debrief-ground = ΕΔΑΦΟΣ
debrief-weapons = ΟΠΛΑ
debrief-radar-ids = ΤΑΥΤΟΠΟΙΗΣΕΙΣ RADAR
debrief-visual-ids = ΟΠΤΙΚΕΣ ΤΑΥΤΟΠΟΙΗΣΕΙΣ
debrief-hits-taken = ΠΛΗΓΜΑΤΑ ΠΟΥ ΔΕΧΘΗΚΑΤΕ
debrief-time = ΧΡΟΝΟΣ
debrief-section-points = Πόντοι
debrief-difficulty = ΔΥΣΚΟΛΙΑ
debrief-total = ΣΥΝΟΛΟ
debrief-section-timeline = Χρονολόγιο
debrief-no-events = Κανένα γεγονός
debrief-section-exposure = Έκθεση σε radar
debrief-tracked = ΠΑΡΑΚΟΛΟΥΘΗΣΗ
debrief-closest = ΠΙΟ ΚΟΝΤΑ
debrief-worst = ΧΕΙΡΟΤΕΡΟ
debrief-at = Στις { $time }
debrief-altitude = ΥΨΟΣ
debrief-speed = ΤΑΧΥΤΗΤΑ
debrief-margin = ΠΕΡΙΘΩΡΙΟ
debrief-strongest = ΙΣΧΥΡΟΤΕΡΟ
debrief-tracked-by = ΠΑΡΑΚΟΛΟΥΘΗΣΗ ΑΠΟ { $sites }
debrief-section-map = Χάρτης
debrief-legend-safe = Μη ανιχνεύσιμος
debrief-legend-close = Κοντά στον εντοπισμό
debrief-legend-detectable = Ανιχνεύσιμος
debrief-legend-tracked = Παρακολούθηση / εκτόξευση πυραύλου
debrief-legend-hit = Πλήγμα που δεχθήκατε / κατάρριψη
map-symbols = Σύμβολα
map-legend-clouds = Σύννεφα / καταιγιδοφόροι πύργοι
map-legend-snow = Χιόνι, από ασθενές ως ισχυρό
map-legend-rain = Βροχή, από ασθενή ως ισχυρή
map-legend-thunderstorm = Καταιγιδοφόρο κύτταρο
debrief-pause = ΠΑΥΣΗ
debrief-play = ΑΝΑΠΑΡΑΓΩΓΗ
