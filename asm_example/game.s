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
  LD V7, 26 // (64/2)-4-2
  LD V8, 14 // (32/2)-2

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
  LD V0, 0
  // Movement speed
  LD V2, 1

  // 5 = W = up
  LD V3, 5
  SKP V3
  JP left 
  SUB V8, V2
  LD V0, 1

left:
  // 7 = A
  LD V3, 7
  SKP V3
  JP down
  SUB V7, V2
  LD V0, 1

down:
  // 8 = S
  LD V3, 8
  SKP V3
  JP right
  ADD V8, V2
  LD V0, 1

right:
  // 9 = D 
  LD V3, 9
  SKP V3
  JP return
  ADD V7, v2
  LD V0, 1

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

  LD V9, 0
  LD V10, 0
  
  // Skip 100s, we'll reset game before then
  LD F, V1
  DRW, V9, V10, 5
  ADD V9, 5

  LD F, V2
  DRW, V9, V10, 5
  ADD V9, 5

  RET

game_win:
  CLS
  LD I, player_data
  // Block X and y
  // Start on end of screen since sprite is only 4x4 
  LD V0, 60
  LD V1, 0
  //Inc amount
  LD V10, 4
  // Timer
  LD V9, 8

draw_start:
  LD DT, V9

draw_timer:
  LD V2, DT
  SE V2, 0
  JP draw_timer

  // Check if we're at the bottom of the sreen
  // Do this here not after moving down a row
  // so that we get to see the last block for
  // a fixed amount of time.
  SNE V1, 32
  // Restart game
  JP setup

  DRW V0, V1, 4
  // Inc X
  ADD V0, V10

  // If we're at the end move down a row
  // 60 + 64, since we're using the overflow
  SE V0, 124 
  JP draw_start

  // Reset X
  LD V0, 60 
  // Inc Y
  ADD V1, V10
  JP draw_start

game:
  CLS
  CALL draw_object
  CALL draw_player
  SE VF, 1
  JP no_hit
  ADD V6, 1
  SNE V6, 11
  JP game_win
  CALL place_object
no_hit:
  CALL draw_score

wait_timer:
  LD VF, DT
  SE, VF, 0
  JP wait_timer
  
  CALL move_player
  SNE V0, 1
  JP game
  JP wait_timer 

end:
  JP end
