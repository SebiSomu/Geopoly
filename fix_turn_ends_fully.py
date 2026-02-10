import os

path = r'c:\proiecte-vue\monopoly-world-edition\backend\game-engine\src\game.rs'
with open(path, 'r', encoding='utf-8') as f:
    text = f.read()

# All variants that should trigger turn_ends = false
full_list = "GameStep::WaitingForPurchaseDecision { .. } | GameStep::WaitingForFirstClassDecision { .. } | GameStep::WaitingForAirportDecision { .. } | GameStep::WaitingForAirportDestination { .. } | GameStep::WaitingForTargetSelection { .. } | GameStep::WaitingForDiceDuel { .. } | GameStep::WaitingForForcedDeal"

import re

# Find any block of GameStep::Waiting... followed by => false
# We'll use a regex that handles line breaks and any GameStep variants
regex = r'(GameStep::WaitingFor[a-zA-Z0-9_{}\.\.\s\|]+)=>\s*false'

text = re.sub(regex, full_list + " => false", text)

with open(path, 'w', encoding='utf-8') as f:
    f.write(text)

print("Done")
