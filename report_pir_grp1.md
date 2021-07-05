# Datenfluss und Struktur
Bei der Datenstruktur gibt es im Wesentlichen drei Knackpunkte. Zum Einen wird fuer jede Aenderung an einer Datei die Zeile mit dem Aenderungsdatum und dem Pfad der Datei gespeichert. Das Speichern uebernimmt eine binaer serialisierte Objekt-aehnliche Datenbank. Desweiteren wird fuer jede Datei ein eigener Versions-Stack erstellt, der Versionen nach Zeitabschnitten lagert. Zum Anderen hat jede Datei einen eigenen Versions-Marker der inkrementiert bzw. dekrementiert wird und somit die Zeitabschnitte der Versionen auswaehlt. Die Datenbank iteriert fuer jede Datei den Versions-Stack mithilfe des jetzigen Versions-Marker und filtert alle Zeilenaenderungen die in den angegebenen Zeitabschnitten und zusaetzlich noch in einem bestimmten (ueber die UI auswaehlbares) Zeitfenster liegen. Es wird also nach Zeitabschnitten und auch einem Zeitfenster gefiltert.
Da alle bestehen Datensaetze (Zeilenaenderungen) stets unveraendert bleiben (immutibility) d.h. fuer jede Aenderung ein neuer Datensatz erstellt wird, ist es sehr einfach die "Hits of Code" Metrik pro Datei zu berechnen. Bei der "Hits of Code" Metrik handelt es sich um ein Mass wie viele Aenderungen an einer Datei insgesamt durchgefuehrt wurden. Das bedeutet es wird nicht auf Zeilenlaenge sondern effektive Zeilenbearbeitung geachtet.
Die "Hits of Code" Metrik wird dabei pro Datei taeglich erstellt.

Der Filewatcher delegiert seine Arbeit bei jeder erfassten Aenderung an den Eventhandler der sozusagen als Middleman zwischen Front- und Backend dient. Dieser speichert naemlich neue Aenderungen in der Datenbank ab, vergleicht bestehende Speicherstaende mit der neuen Aenderung ueber ein Differenz-Algorithmus und schickt diese letztlich an das Frontend zur visuellen Ausgabe.
Davon ausgenommen sind Formatkonvertierungen, um das resultierende Ergebnis von der Rust-internen Datenstruktur auf die Datenstruktur umzubauen, welche für die Darstellung in der TUI benötigt wird.

# Umsetzung
Die Umsetzung des Projektes erfolgte nach dem Konzept, zuerst ein Grundgerüst aufzubauen, welches minimale Abhängigkeiten aufweist. Das heißt, das Programm startet, bemerkt Dateiänderungen, registriert diese. Der zweite große Schritt besteht darin, nötige Datenstrukturen aufzusetzen. Ab diesem Punkt liess sich die Entwicklung zweigleisig betreiben, so dass sowohl Frontend und Backend unabhängig voneinander entwickelt werden konnten. Um hierbei eine systematische Arbeitsweise möglich zu machen, wurde auf GitHub Issues zurückgegriffen, die dann granular sowohl in zu erreichende Ziele, als auch auf dem Weg aufgetretene Fehler dokumentiert wurden.
Die Umsetzung ging natürlich freilich nicht so geschmeidig vonstatten wie es sich liest, da viel des Know-how's erst auf dem Weg gelernt werden musste. Vergleichbar ist das so, als wenn man einen Film drehen möchte, dabei aber noch die Kamera entwickeln muss. Dazu in folgenden mehr.

# Probleme und Schwierigkeiten
Ein Problem war unter anderem, die Kommunikation zwischen Backend und Frontend herzustellen. Um eine saubere Lösung zu bekommen, wurde hier auf Channels zurückgegriffen. Allerdings hat dies zu dem Problem geführt, da ein Listener-Channel in einem eigenen Thread laufen muss. Dies ist in Rust nicht ohne Weiteres umzusetzen, da dieses Modul nicht sicher unter Threads geteilt werden kann. Dieses nun exemplarisch herausgezogene Problem lässt sich allgemein als Thread-Concurrency-Problem beschreiben, welches uns durch die gesamte Entwicklung hinweg begleitet hat. Die Lösung dieser Probleme erfolgte durch intensive Recherche und Nachforschung, unter Anderem auch mal ein wenig in der Rust Community nachgehakt.
Die nächste Herausforderung erwuchs im Umgang mit der TUI, weil diese Art der Frontend-Entwicklung sehr unintuitiv ist und ebenso die Dokumentation letzteres zu wünschen übrig lässt. Ein systematischer Ansatz war nicht gegeben, sodass nach dem Try & Error Prinzip gearbeitet werden musste, um überhaupt die TUI kennenlernen zu können. Das hat dazu geführt, das gewisse Arbeitsstaende mehrmals neu gemacht werden mussten, und schlussendlich eine stabile Ausarbeitung auf die Beine stellen zu können.
Es hat sich herausgestellt, dass unter Windows ein Greedy-Algorithmus was die Freigabe von Mutex Locks angeht. Daher musste von der Standard-Bibliothek abgewichen werden und ein externes crate verwendet werden.
Auch bei den Channels musste von der Standard-Bibliothek abgewichen werden, da die Sender nicht klonbar sind.
# Was es kann
Bis jetzt kann das Tool eben implizit Aenderungen abspeichern und diese zuruecksetzen/wiederherstellen.
Dies unterstuetzt zusaetzlich eine grafische Oberflaeche die noch die "Hits-of-Code" Metrik anzeigt.
Ausserdem kann man in einer Konfigurationsdatei auswaehlen welche Pfade und Dateien vom ganzen Ueberwachungsprozess ausgeschlossen sind.


# Hinweise zum Start
Zum Starten muss die config.toml um den watch-path und den store-path ergaenzt werden. Es koennen auch 
Dateien angegeben werden, die ignoriert werden sollen. Die Datei ist mit einem Beispiel versehen. Vor dem
Starten bitte anpassen!
