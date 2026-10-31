# Τα μηνύματα του παιχνιδιού στο HUD και στα 2D επίπεδα της πτήσης (src/hud, src/combat.rs,
# src/csar_view.rs, src/campaign). Το HUD τα σχεδιάζει με κεφαλαία χωρίς τόνους· οι όροι
# της αβιονικής (CCIP, THR, KM, MAX...) μένουν στα αγγλικά.

hud-no-ejection-seat = ΧΩΡΙΣ ΚΑΘΙΣΜΑ ΕΚΤΙΝΑΞΗΣ
hud-waypoint-reached = Το waypoint { $waypoint } επιτεύχθηκε
hud-select-camera = ΕΠΙΛΟΓΗ ΚΑΜΕΡΑΣ
hud-open-bay = ΑΝΟΙΞΤΕ ΤΟ ΚΑΜΠΑΡΤΙΝΟ
hud-camera-page = ΣΕΛΙΔΑ { $page }
hud-target-in-view-shoot = ΣΤΟΧΟΣ ΣΤΟ ΟΠΤΙΚΟ ΠΕΔΙΟ - ΦΩΤΟΓΡΑΦΙΑ
hud-target-in-view = ΣΤΟΧΟΣ ΣΤΟ ΟΠΤΙΚΟ ΠΕΔΙΟ
hud-aim-at-target = ΣΗΜΑΔΕΨΤΕ ΤΟΝ ΣΤΟΧΟ
hud-photo = ΦΩΤΟ { $state }
hud-rearmed = Επανοπλίστηκε και ανεφοδιάστηκε
hud-landed-rearm = Προσγειωθήκατε - επανοπλίστε ή τερματίστε την εξόρμηση
hud-objective-achieved = Ο στόχος επιτεύχθηκε

## Οθόνη τέλους αποστολής

end-accomplished = επιτεύχθηκε
end-not-accomplished = δεν επιτεύχθηκε
end-primary = Κύριος στόχος: { $result }
end-secondary = Δευτερεύων στόχος: { $result }
end-kills = Καταρρίψεις: { $air } αεροσκάφη, { $ground } στόχοι εδάφους
end-time = Διάρκεια αποστολής { $time }
end-damage = Ζημιές: { $hits } πλήγματα
end-training = Αποστολή εκπαίδευσης
end-press-any-key = Πατήστε ένα πλήκτρο

## Το πάνελ του καταρριφθέντος πιλότου

csar-flare = ΦΩΤΟΒΟΛΙΔΑ ΣΗΜΑΤΟΣ
csar-faster = ΠΙΟ ΓΡΗΓΟΡΑ
csar-normal = ΚΑΝΟΝΙΚΟΣ ΧΡΟΝΟΣ
csar-skip = ΠΑΡΑΛΕΙΨΗ ΔΙΑΣΩΣΗΣ
csar-parachute = ΑΝΟΙΧΤΟ ΑΛΕΞΙΠΤΩΤΟ
csar-on-ground = ΣΤΗ ΓΗ
csar-down = ΚΑΤΩ ΣΕ ΕΧΘΡΙΚΟ ΕΔΑΦΟΣ
csar-pickup = ΔΙΑΣΩΣΗ   UH-60 ΑΠΟ ΠΑΝΩ - ΑΝΑΛΗΨΗ
csar-inbound = ΔΙΑΣΩΣΗ   UH-60 ΣΕ { $km } KM { $direction } - ΠΡΟΣΕΓΓΙΖΕΙ
csar-fire-flare = ΔΙΑΣΩΣΗ   UH-60 ΣΕ { $km } KM { $direction } - ΡΙΞΤΕ ΦΩΤΟΒΟΛΙΔΑ ΟΤΑΝ ΠΛΗΣΙΑΣΕΙ
csar-enemy = ΕΧΘΡΟΣ    { $count } HIND, ΠΛΗΣΙΕΣΤΕΡΟ ΣΕ { $km } KM
csar-flares = ΦΩΤΟΒΟΛΙΔΕΣ   { $count }      ΣΤΗ ΓΗ { $time }

## Ειδοποιήσεις συνεργασίας στο πάνελ (src/player_notices.rs), σημείωση της κονσόλας αεράμυνας

notice-joined = Ο/Η { $pilot } συνδέθηκε
notice-left = Ο/Η { $pilot } αποχώρησε
notice-lost-connection = Ο/Η { $pilot } έχασε τη σύνδεση
st-select-track = Επιλέξτε πρώτα ένα ίχνος
