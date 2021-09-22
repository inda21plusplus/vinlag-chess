# vinlag-chess
Very fancy libary with such good CLI rendering that it does not need a GUI.

If you want to see a working example on how to use the code see https://gits-15.sys.kth.se/inda21plusplus/vinlag-chess/blob/master/src/main.rs

**init_game_board** to create a game using FEN, you can pass **STANDARD_BOARD** for a standard game

**get_game_state** to see if someone has won the game or it is a tie

**parse_move** to parse input in e6e7 format, used for CLI

**move_piece** to move a piece, will return true if the piece was moved

**get_all_valid_moves** will return all positions a piece can move and can be used to display a map of possible moves

**get_threats** will return metadata that is used for debugging and for other functions to work, use it with **get_game_state** and **move_piece**

**get_fen** will return a FEN string of the current game, use to export

**promote_pawn** will return true if the pawn was promoted, only add this if **move_piece** **auto_promote = false** and you need to keep track of this yourself as any next move will be counted as invalid if a pawn is not promoted.

**render** this will render the game in the terminal
