import os
import re

path = r'c:\proiecte-vue\monopoly-world-edition\backend\game-engine\src\game.rs'
with open(path, 'r', encoding='utf-8') as f:
    text = f.read()

# The clean block we want
# Note: I'll use the indentation from the file
clean_block = """let turn_ends = match self.step {
                        GameStep::WaitingForPurchaseDecision { .. } |
                        GameStep::WaitingForFirstClassDecision { .. } |
                        GameStep::WaitingForAirportDecision { .. } |
                        GameStep::WaitingForAirportDestination { .. } |
                        GameStep::WaitingForTargetSelection { .. } |
                        GameStep::WaitingForDiceDuel { .. } |
                        GameStep::WaitingForForcedDeal => false,
                        _ => true,
                    };"""

# Use re.DOTALL to match over line breaks
# Use a non-greedy .*? to find until the first _ => true, ... };
pattern = r'let turn_ends = match self\.step \{.*?_ => true,.*?\};'

text = re.sub(pattern, clean_block, text, flags=re.DOTALL)

with open(path, 'w', encoding='utf-8') as f:
    f.write(text)

print("Done")
