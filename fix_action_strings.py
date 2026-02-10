import os

path = r'c:\proiecte-vue\monopoly-world-edition\backend\game-engine\src\game.rs'
with open(path, 'r', encoding='utf-8') as f:
    text = f.read()

# Change manual action strings to PascalCase to match HereAndNow variants and UI expectations
text = text.replace('"dice_challenge"', '"DiceDuel"')
text = text.replace('"swap_stamps"', '"SwapStamps"')
# Note: SneakySwap is handled by handle_forced_deal usually, but let's check
text = text.replace('"sneaky_swap"', '"SneakySwap"')

with open(path, 'w', encoding='utf-8') as f:
    f.write(text)

print("Done")
