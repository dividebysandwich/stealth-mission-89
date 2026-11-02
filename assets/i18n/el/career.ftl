# Σταδιοδρομίες πιλότων (src/career.rs): βαθμοί, παράσημα, περίοδος υπηρεσίας.

## Βαθμοί της Πολεμικής Αεροπορίας (αποδίδονται με τους ισοδύναμους), συντμήσεις

rank-0 = Ανθυποσμηναγός
    .short = Ανθσμγός
rank-1 = Υποσμηναγός
    .short = Υποσμγός
rank-2 = Σμηναγός
    .short = Σμηναγός
rank-3 = Επισμηναγός
    .short = Επισμγός
rank-4 = Αντισμήναρχος
    .short = Αντισμχος
rank-5 = Σμήναρχος
    .short = Σμχος

## Παράσημα (αμερικανικά μετάλλια: τα κύρια ονόματά τους)

medal-air-medal = Air Medal
medal-distinguished-flying-cross = Distinguished Flying Cross
medal-silver-star = Silver Star
medal-air-force-cross = Air Force Cross
medal-medal-of-honor = Medal of Honor
medal-purple-heart = Purple Heart
medal-prisoner-of-war-medal = Prisoner of War Medal

## Νέα μετά από εξόρμηση

career-awarded = Απονομή: { $medal }
career-kia = Φονεύθηκε εν ώρα μάχης
    .short = KIA
career-pow = Αιχμάλωτος πολέμου
    .short = POW
career-active = Σε ενεργό υπηρεσία
career-captured = Αιχμαλωσία: η σταδιοδρομία τελείωσε
career-tour-sortie = Περίοδος υπηρεσίας: εξόρμηση { $sortie } από { $length }
career-tour-complete = Η περίοδος υπηρεσίας ολοκληρώθηκε: { $points } πόντοι (+{ $bonus })
career-promoted = Προαγωγή: { $rank }

## Το παράθυρο σταδιοδρομίας

career-title = ΜΗΤΡΩΟ ΠΙΛΟΤΩΝ
career-group-tour = Περίοδος
career-set-by-tour = Ορίζεται από την περίοδο υπηρεσίας του πιλότου
career-pilots = Πιλότοι
career-new-pilot = Νέος πιλότος
career-name = Όνομα
career-enlist = ΣΤΡΑΤΟΛΟΓΗΣΗ
career-select = Επιλέξτε ή στρατολογήστε έναν πιλότο
career-service-record = Μητρώο υπηρεσίας
career-rank = ΒΑΘΜΟΣ
career-points = ΠΟΝΤΟΙ
career-sorties = ΕΞΟΡΜΗΣΕΙΣ
career-tours = ΠΕΡΙΟΔΟΙ
career-status = ΚΑΤΑΣΤΑΣΗ
career-decorations = Παράσημα
career-tour-of-duty = Περίοδος υπηρεσίας
career-tour-line = { $theater }  { $situation }  εξόρμηση { $sortie } από { $length }  { $points } πόντοι
career-destroyed = { $count } στόχοι καταστράφηκαν μέχρι τώρα
career-fly-next = ΕΠΟΜΕΝΗ ΕΞΟΡΜΗΣΗ
career-new-tour = { $count } εξορμήσεις, { $theater }, { $situation }, αντίπαλοι: { $opponents }
    .facts = Ό,τι καταστρέφετε μένει κατεστραμμένο μέχρι το τέλος της περιόδου υπηρεσίας
career-start-tour = ΕΝΑΡΞΗ ΠΕΡΙΟΔΟΥ
career-retired = Αποστρατεία από την ενεργό υπηρεσία
career-last-sorties = Τελευταίες εξορμήσεις
