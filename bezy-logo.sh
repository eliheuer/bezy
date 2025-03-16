#!/bin/bash

# ANSI color code for green
GREEN='\033[0;32m'
# ANSI color code to reset to default
RESET='\033[0m'

# Function to print text in green
print_green() {
    echo -e "${GREEN}$1${RESET}"
}

# Custom ASCII art
print_green "       BBBBBBBBBB          BBBBBBBBBBBBBBB"
print_green "   BBBBBBBBBBBBBBBBB       BBBBBBBBBBBBBBBB"
print_green "  BBBB           BBBBB     BBB           BBB"
print_green " BBB    BBBBBBB    BBBB    BBB           BBBB        BBBBBBBB       BBBBBBBBBBBBB  BBB         BBB"
print_green "BBBB   BBB  BBBBB   BBBB   BBB           BBB      BBBBBBBBBBBBB     BBBBBBBBBBBBB  BBB         BBB"
print_green "BBB    BBB    BBBB   BBB   BBBBBBBBBBBBBBBB      BBBB        BBBB           BBBB   BBB         BBB"
print_green "BBBB    BBBB   BBB   BBB   BBBBBBBBBBBBBBBB     BBBB          BBB          BBB      BBB       BBB"
print_green " BBBB         BBBB   BBB   BBB          BBBB    BBBBBBBBBBBBBBBBB        BBBB        BBB     BBB"
print_green "   BBBBBBBBBBBBB    BBBB   BBB            BBBB  BBBBBBBBBBBBBBBBB       BBB           BBB   BBB"
print_green "      BBBBBBBB     BBBB    BBB             BBB  BBBB                  BBBB             BBB BBB"
print_green "BB               BBBBB     BBB            BBBB   BBBB         BBB    BBB                BBBBB"
print_green "BBBBBB       BBBBBBB       BBBBBBBBBBBBBBBBBB     BBBBBBBBBBBBBB   BBBBBBBBBBBBBB        BBB"
print_green " BBBBBBBBBBBBBBBBB         BBBBBBBBBBBBBBBB         BBBBBBBBBB     BBBBBBBBBBBBBB       BBB"
print_green "     BBBBBBBBB                                                                         BBB"
print_green "                                                                                    BBBBB"
print_green "A Font.Garden Tool                                                                  BBBB"
