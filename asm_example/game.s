  JP setup

player_data:
  // 4x4 player sprite
  .word 0x0F0F
  .word 0x0F0F

object_data:
  .word 0x1000

score_temp: 
  // To store BCD
  .word 0x0000
  .word 0x0000

draw_player:
  LD I, player_data
  DRW, V7, V8, 4
  RET

setup:
  //Put player in center of screen
  //(set pixels are on the right side of the sprite) 
  LD V7, 0x1A // (64/2)-4-2 = 26 = 0x1A
  LD V8, 0x0E // (32/2)-2 = 14 = 0xE

  //Timer reset value
  LD V10, 0xF0
  LD DT, V10

  //Score
  LD V6, 0x00

  CALL place_object

  JP game

place_object:
  RND V4, 0x3F
  RND V5, 0x1F
  RET

draw_object:
  LD I, object_data
  DRW V4, V5, 1
  RET

move_player:
  // Return flag, =1 if we moved
  LD V0, 0x00
  // Movement speed
  LD V2, 0x1

  // 5 = W = up
  LD V3, 0x5
  SKP V3
  JP left 
  SUB V8, V2
  LD V0, 0x1

left:
  // 7 = A
  LD V3, 0x7
  SKP V3
  JP down
  SUB V7, V2
  LD V0, 0x1

down:
  // 8 = S
  LD V3, 0x8
  SKP V3
  JP right
  ADD V8, V2
  LD V0, 0x1

right:
  // 9 = D 
  LD V3, 0x9
  SKP V3
  JP return
  ADD V7, v2
  LD V0, 0x1

return:
  //Reset timer
  LD DT, V10
  RET

draw_score:
  LD I, score_temp
  // Writes BCD to memory
  LD B, V6
  // Get it back in parts
  LD V2, [I]
  // Get digits and draw

  LD V9, 0x00
  LD V10, 0x00
  
  // Skip 100s, we'll reset game before then
  LD F, V1
  DRW, V9, V10, 5
  ADD V9, 0x5

  LD F, V2
  DRW, V9, V10, 5
  ADD V9, 0x5

  RET

game_win:
  CLS
  LD I, player_data
  // Block X and y
  // Start on end of screen since sprite is only 4x4 
  LD V0, 0x3c 
  LD V1, 0x00
  //Inc amount
  LD V10, 0x4
  // Timer
  LD V9, 0x8
draw_start:
  LD DT, V9
draw_timer:
  LD V2, DT
  SE V2, 0x0
  JP draw_timer
  DRW V0, V1, 4
  ADD V0, V10
  // 60 + 64 = 124, since we're using the overflow
  SE V0, 0x7c
  JP draw_start
  // Reset X
  LD V0, 0x3c
  // Move down a row
  ADD V1, V10
  // Check if we're at the bottom of the sreen
  SNE V1, 0x20
  // Restart game
  JP setup
  JP draw_start

game:
  CLS
  CALL draw_object
  CALL draw_player
  SE VF, 0x01
  JP no_hit
  ADD V6, 0x1
  SNE V6, 0xB
  JP game_win
  CALL place_object
no_hit:
  CALL draw_score

wait_timer:
  LD VF, DT
  SE, VF, 0x0
  JP wait_timer
  
  CALL move_player
  SNE V0, 0x1
  JP game
  JP wait_timer 

end:
  JP end
