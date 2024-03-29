# Define color options and display exit status
bold=$(tput bold)
normal=$(tput sgr0)
BLACK=$(tput setaf 0)
RED=$(tput setaf 1)
GREEN=$(tput setaf 2)
YELLOW=$(tput setaf 3)
BLUE=$(tput setaf 4)
MAGENTA=$(tput setaf 5)
CYAN=$(tput setaf 6)
WHITE=$(tput setaf 7)
DEFAULT_COLOR=$(tput setaf 9)

PROJECT_NAME='chip8'
CLEAR_FLAG=false
TEST_FLAG=false
DEBUG_MSG_FLAG=''
HELP_FLAG=''
FILENAME=''

while [[ $# -gt 0 ]]; do 
    case $1 in 
        -c|--clear) 
            CLEAR_FLAG=true
            shift
            ;; 
        -d|--debug) 
            DEBUG_MSG_FLAG='-d'
            shift
            ;; 
        -h|--help) 
            HELP_FLAG='-h'
            shift
            ;;
        -t|--test) 
            TEST_FLAG=true
            shift
            ;; 
        -*|--*) 
            echo "BUILD SCRIPT:: unknown option $1"
            exit 1
            ;; 
        *) 
            FILENAME="$1"
            shift
            ;; 
    esac
done

if [ $TEST_FLAG = true ] ; then  
    clear
    cargo test 
    exit 0
fi 

# build the code changes
clear
cargo build

# clear output (should be used if build is stable and no warnings are generated)
if [ $CLEAR_FLAG = true ] ; then 
    clear
fi

# run executable
echo 'STDOUT:'
if [ "$FILENAME" = "" ] ; then 
    ./target/debug/$PROJECT_NAME $DEBUG_MSG_FLAG $HELP_FLAG
else 
    ./target/debug/$PROJECT_NAME $DEBUG_MSG_FLAG $HELP_FLAG "$FILENAME"
fi

# check exit status
exit_code=$? 
if [ $exit_code -eq 0 ]; then 
    echo "\nFinished with exit code: ${bold}${GREEN}$exit_code${normal}"
else
    echo "\nFinished with exit code: ${bold}${RED}$exit_code${normal}"
fi 