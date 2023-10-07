tell app "System Events"
    activate
    set chosenFile to choose file
    set posix_path to POSIX path of chosenFile
end tell

posix_path 