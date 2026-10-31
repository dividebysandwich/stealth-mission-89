# Ο επεξεργαστής οπλισμού (src/loadout.rs). Τα ονόματα των όπλων είναι τα πραγματικά, αμετάφραστα.

loadout-none = Κανένα
loadout-briefed = ΟΠΩΣ ΣΤΟ BRIEFING
    .facts = Τα φορτία με τα οποία σχεδιάστηκε η αποστολή
loadout-clear = ΕΚΚΑΘΑΡΙΣΗ
    .facts = Όλα τα σημεία φόρτωσης άδεια
loadout-done = OK
loadout-no-hardpoints = Κανένα σημείο φόρτωσης
loadout-no-stores = Αυτό το αεροσκάφος δεν φέρει φορτία
loadout-fantasy = Δεν υπάρχει στο πραγματικό αεροσκάφος
loadout-empty = Άδειο
loadout-out-of-stock = Δεν υπάρχει απόθεμα για αυτή την αποστολή
loadout-mirror = Ίδιο και στις δύο πλευρές
    .facts = Φορτώνει και το { $station }
loadout-stores = Φορτία
loadout-needs-equipment = Η αποστολή χρειάζεται την κάμερα ή τον ειδικό εξοπλισμό της

## Ο ρόλος κάθε όπλου (τύποι όπλων 0 έως 20)

# AIM-9M Sidewinder
weapon-role-0 = Αέρος-αέρος, καθοδήγηση υπερύθρων, μικρή εμβέλεια
# AIM-120A AMRAAM
weapon-role-1 = Αέρος-αέρος, καθοδήγηση radar, μέση εμβέλεια
# AGM-88A HARM
weapon-role-2 = Αντιραδιοβολιακός πύραυλος: radar και θέσεις SAM
# Penguin 3 ASM
weapon-role-3 = Αντιπλοϊκός πύραυλος
# AGM-84A Harpoon
weapon-role-4 = Αντιπλοϊκός πύραυλος μεγάλης εμβέλειας
# AGM-65 Maverick
weapon-role-5 = Πύραυλος με τηλεοπτική καθοδήγηση: οχήματα, καταφύγια, κτίρια
# GBU-12 Paveway
weapon-role-6 = Βόμβα με καθοδήγηση λέιζερ: ενισχυμένοι στόχοι
# Mk 20 Rockeye
weapon-role-7 = Βόμβα διασποράς: μαλακοί στόχοι και στόχοι περιοχής
# Durandal
weapon-role-8 = Βόμβα καταστροφής διαδρόμου
# Mk 82 LDGP
weapon-role-9 = Βόμβα ελεύθερης πτώσης 500 lb
# Mk 82 Snakeye
weapon-role-10 = Επιβραδυνόμενη βόμβα 500 lb για ρίψεις από χαμηλό ύψος
# Mk 20 Rockeye II
weapon-role-11 = Βόμβα διασποράς: τεθωρακισμένα και στόχοι περιοχής
# Mk 122 Fireye
weapon-role-12 = Εμπρηστική βόμβα
# CBU-72 FAE
weapon-role-13 = Εκρηκτικό καυσίμου-αέρα: στόχοι περιοχής
# Mk 35 incendiary
weapon-role-14 = Εμπρηστική βόμβα διασποράς
# ISCB-1 mines
weapon-role-15 = Νάρκες άρνησης περιοχής
# IR/optical camera
weapon-role-16 = Φωτογραφίες αναγνώρισης του στόχου
# 1900 lb fuel tank
weapon-role-17 = Επιπλέον καύσιμο, μεταφέρεται κατά τη ρίψη
# Special cargo
weapon-role-18 = Εφόδια ή φορτίο για την αποστολή
# 20 mm cannon
weapon-role-19 = Το πυροβόλο (δεν είναι φορτίο)
# AGM-114 Hellfire
weapon-role-20 = Αντιαρματικός πύραυλος με καθοδήγηση λέιζερ: πετά εκεί που κοιτά το targeting pod

## Πώς χρησιμοποιείται κάθε όπλο (εμβέλειες σε km του HUD, ύψη σε πόδια)

employment-0 = Εμβέλεια: 1-12 km, μεγαλύτερη κατά μέτωπο
employment-1 = Εμβέλεια: 1-38 km, μεγαλύτερη κατά μέτωπο
employment-2 =
    Εμβέλεια: 9-50 km
    Εκτόξευση: 5.000-20.000 ft
employment-3 =
    Εμβέλεια: 6-14 km (κλασική καθοδήγηση: 13-14 km)
    Εκτόξευση: 100-500 ft
employment-4 =
    Εμβέλεια: 6-41 km
    Εκτόξευση: 2.000-5.000 ft
employment-5 =
    Εμβέλεια: 5-10 km
    Εκτόξευση: 2.000-5.000 ft
employment-6 =
    Εμβέλεια: από 1,5 km έως 2,5 φορές το ύψος, LOCK σταθερό (14 km από 15.000 ft, 18 km από 25.000 ft)
    Ρίψη: 15.000-25.000 ft
employment-8 =
    Εμβέλεια: 0,9-1,3 km πριν τον διάδρομο
    Ρίψη: 200-500 ft
employment-9 =
    Εμβέλεια: 5-11 km πριν τον στόχο
    Ρίψη: 3.000-20.000 ft
employment-12 =
    Εμβέλεια: 0,9-2,2 km πριν τον στόχο
    Ρίψη: 100-500 ft
employment-16 = Εμβέλεια: φωτογραφίες από 2-8 km (όσο πιο κοντά, τόσο περισσότεροι αντίπαλοι), μύτη προς τον στόχο
employment-18 = Εμβέλεια: ρίψη στο "Drop supplies now", σε απόσταση έως 4 km από τη ζώνη ρίψης
employment-20 =
    Εμβέλεια: 4-8 km
    Εκτόξευση: 2.000-3.000 ft
employment-guided-cluster =
    Εμβέλεια: από 1,5 km έως 2,5 φορές το ύψος, LOCK σταθερό (3,7 km από 3.000 ft, 6 km από 5.000 ft)
    Ρίψη: 3.000-5.000 ft
employment-medium-retarded =
    Εμβέλεια: 1,1-1,6 km πριν τον στόχο
    Ρίψη: 300-1.000 ft
employment-low-retarded =
    Εμβέλεια: 0,9-1,3 km πριν τον στόχο
    Ρίψη: 200-500 ft

## Σημεία φόρτωσης

hardpoint-left-bay = Αριστερός θάλαμος
hardpoint-right-bay = Δεξιός θάλαμος
hardpoint-left-wing-bay = Θάλαμος αριστερής πτέρυγας
hardpoint-right-wing-bay = Θάλαμος δεξιάς πτέρυγας
hardpoint-station-1-left-wingtip = Θέση 1, αριστερό ακροπτερύγιο
hardpoint-station-2-left-outer-wing = Θέση 2, αριστερή πτέρυγα εξωτερικά
hardpoint-station-3-left-wing = Θέση 3, αριστερή πτέρυγα
hardpoint-station-4-left-inner-wing = Θέση 4, αριστερή πτέρυγα εσωτερικά
hardpoint-station-5-centreline = Θέση 5, κεντρική γραμμή
hardpoint-station-6-right-inner-wing = Θέση 6, δεξιά πτέρυγα εσωτερικά
hardpoint-station-7-right-wing = Θέση 7, δεξιά πτέρυγα
hardpoint-station-8-right-outer-wing = Θέση 8, δεξιά πτέρυγα εξωτερικά
hardpoint-station-9-right-wingtip = Θέση 9, δεξί ακροπτερύγιο
hardpoint-station-1-left-wingtip-rail = Θέση 1, ράγα αριστερού ακροπτερυγίου
hardpoint-station-2-left-outer-pylon = Θέση 2, αριστερός εξωτερικός πυλώνας
hardpoint-station-3-left-inner-pylon = Θέση 3, αριστερός εσωτερικός πυλώνας
hardpoint-station-4-right-inner-pylon = Θέση 4, δεξιός εσωτερικός πυλώνας
hardpoint-station-5-right-outer-pylon = Θέση 5, δεξιός εξωτερικός πυλώνας
hardpoint-station-6-right-wingtip-rail = Θέση 6, ράγα δεξιού ακροπτερυγίου
hardpoint-tornado-left-wing-rail = Αριστερή πτέρυγα, ράγα Sidewinder
hardpoint-tornado-left-wing-pylon = Πυλώνας αριστερής πτέρυγας
hardpoint-tornado-left-shoulder-forward = Αριστερός ώμος, εμπρός
hardpoint-tornado-left-shoulder-aft = Αριστερός ώμος, πίσω
hardpoint-tornado-right-shoulder-aft = Δεξιός ώμος, πίσω
hardpoint-tornado-right-shoulder-forward = Δεξιός ώμος, εμπρός
hardpoint-tornado-right-wing-pylon = Πυλώνας δεξιάς πτέρυγας
hardpoint-tornado-right-wing-rail = Δεξιά πτέρυγα, ράγα Sidewinder
hardpoint-a10-1-left-outer-wing = Θέση 1, αριστερή πτέρυγα εξωτερικά
hardpoint-a10-2-left-wing = Θέση 2, αριστερή πτέρυγα
hardpoint-a10-3-left-wing = Θέση 3, αριστερή πτέρυγα
hardpoint-a10-4-left-centre-wing = Θέση 4, αριστερή πτέρυγα κέντρο
hardpoint-a10-5-left-fuselage = Θέση 5, αριστερή άτρακτος
hardpoint-a10-6-centreline = Θέση 6, κεντρική γραμμή
hardpoint-a10-7-right-fuselage = Θέση 7, δεξιά άτρακτος
hardpoint-a10-8-right-centre-wing = Θέση 8, δεξιά πτέρυγα κέντρο
hardpoint-a10-9-right-wing = Θέση 9, δεξιά πτέρυγα
hardpoint-a10-10-right-wing = Θέση 10, δεξιά πτέρυγα
hardpoint-a10-11-right-outer-wing = Θέση 11, δεξιά πτέρυγα εξωτερικά
hardpoint-av8b-1-left-rail = Θέση 1, αριστερή ράγα Sidewinder
hardpoint-av8b-3-left-middle-wing = Θέση 3, αριστερή πτέρυγα, μέση
hardpoint-av8b-4-left-inboard-wing = Θέση 4, αριστερή πτέρυγα, εσωτερική
hardpoint-av8b-6-right-inboard-wing = Θέση 6, δεξιά πτέρυγα, εσωτερική
hardpoint-av8b-7-right-middle-wing = Θέση 7, δεξιά πτέρυγα, μέση
hardpoint-av8b-9-right-rail = Θέση 9, δεξιά ράγα Sidewinder
weapon-role-21 = Air-to-air, infrared homing, very short range
employment-21 = Range: 1-6 km, farther head-on
