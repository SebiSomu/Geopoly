import os
import re

path = r'c:\proiecte-vue\monopoly-world-edition\backend\game-engine\src\game.rs'
with open(path, 'r', encoding='utf-8') as f:
    text = f.read()

# Pattern for methods that need the fix
methods = [
    'pub fn resolve_purchase',
    'pub fn resolve_first_class',
    'pub fn resolve_target_selection',
    'pub fn resolve_dice_duel'
]

for method in methods:
    start_idx = text.find(method)
    if start_idx == -1: continue
    
    # Find the end of this function (not perfect but usually good for our structure)
    # We find the next 'pub fn' or end of string
    next_func = text.find('pub fn', start_idx + len(method))
    if next_func == -1: next_func = len(text)
    
    func_text = text[start_idx:next_func]
    
    # Identify the relevant index variable name in this function
    idx_var = 'player_idx'
    if 'buyer_idx' in func_text and method == 'pub fn resolve_first_class':
        idx_var = 'buyer_idx'
    elif 'selector_idx' in func_text and method == 'pub fn resolve_target_selection':
        idx_var = 'selector_idx'
    elif 'challenger_idx' in func_text and method == 'pub fn resolve_dice_duel':
        idx_var = 'challenger_idx'
        
    # Replace end_turn with guarded version
    # Avoid double guarding if already guarded
    old_call = 'self.end_turn();'
    new_call = f'if {idx_var} == self.current_player_idx {{ self.end_turn(); }}'
    
    # We want to catch instances that are NOT already guarded
    pattern = rf'(?<!if {idx_var} == self.current_player_idx {{ )self\.end_turn\(\);'
    func_text = re.sub(pattern, new_call, func_text)
    
    text = text[:start_idx] + func_text + text[next_func:]

with open(path, 'w', encoding='utf-8') as f:
    f.write(text)

print("Done")
