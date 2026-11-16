# RLines

RLines is a graphical turn based game.

## Game rules:

* Game is played on a gid of 9 by 9 tiles. This grid is known as game board.
* Goal of the game is to maximize user score.
* User score starts from 0 and is shown in the right upper corner above the board.
* Application and the user take turns with application making a first turn right when the application starts.
* When application takes its turn it places 3 balls at random available tiles.
* If application is able to place 3 all three balls it must place as many as it can.
* If after application takes its turn there are no more available (empty) tiles, then game ends and a modal dialog is displayed with final score. 
* When application ends it turn is select three more random balls (of three random types) to be placed on the board on the next application turn. These balls are shown in a row right above the game grid. 
* One tile can accept only one ball.
* When user takes it turn user selects one ball currently present on the board and then selects empty tile to which user wants to move the ball. 
  * Ball is then moved to that target empty tile if path to that tile is available.
  * After ball is moved by the user if there are lines of balls (horizontal, vertical or diagonal) of the same color with 5 or more balls in the line, then those balls are remove and use score is incremented by the number of removed balls.
  * If no path is available to move selected ball to the target tile, then nothing happens and user is free to either select a different ball or choose a different target tile. 
* When ball is selected it becomes highlighted. User can select another ball, deselecting previously selected one.
* There are three different types of balls, each type with its own color. These colors (and thus types) are: red, purple, green, brown, lightblue and blue.

## Implementation details

* Game is written in Rust and uses `equi` library to render graphics.
* Executable is named `lines`
* Background color for the board must be selectable between black and white. Default background color is black.