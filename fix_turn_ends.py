import os

path = r'c:\proiecte-vue\monopoly-world-edition\backend\game-engine\src\game.rs'
with open(path, 'r', encoding='utf-8') as f:
    text = f.read()

# New variants to add
new_variants = "GameStep::WaitingForTargetSelection { .. } | GameStep::WaitingForDiceDuel { .. } | "

# Fix matches clauses
text = text.replace(
    'GameStep::WaitingForPurchaseDecision { .. } |',
    new_variants + 'GameStep::WaitingForPurchaseDecision { .. } |'
)

# Fix matches! macro at line 248
text = text.replace(
    'GameStep::WaitingForAirportDestination { .. }\n                );',
    'GameStep::WaitingForAirportDestination { .. } |\n                    GameStep::WaitingForTargetSelection { .. } |\n                    GameStep::WaitingForDiceDuel { .. }\n                );'
)

# Fix possible double insertions if I'm not careful, but let's check current state
# Actually, GameStep::WaitingForPurchaseDecision { .. } | appears 6 times in matches
# and once in handles. 

with open(path, 'w', encoding='utf-8') as f:
    f.write(text)

print("Done")
