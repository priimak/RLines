use eframe::egui::{self, Color32, Pos2, Rect, Sense, Stroke, Vec2, Ui};
use eframe::Frame;
use std::collections::VecDeque;

const BOARD_SIZE: usize = 9;
const TILE_SIZE: f32 = 50.0;
const BALL_RADIUS: f32 = 20.0;
const PREVIEW_BALL_RADIUS: f32 = 18.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum BallColor {
    Red,
    Purple,
    Green,
    Brown,
    LightBlue,
    Blue,
}

impl BallColor {
    fn all() -> [BallColor; 6] {
        [
            BallColor::Red,
            BallColor::Purple,
            BallColor::Green,
            BallColor::Brown,
            BallColor::LightBlue,
            BallColor::Blue,
        ]
    }

    fn to_color32(self, _dark_mode: bool) -> Color32 {
        match self {
            BallColor::Red => Color32::from_rgb(220, 50, 50),
            BallColor::Purple => Color32::from_rgb(160, 60, 200),
            BallColor::Green => Color32::from_rgb(50, 180, 70),
            BallColor::Brown => Color32::from_rgb(180, 120, 50),
            BallColor::LightBlue => Color32::from_rgb(60, 180, 220),
            BallColor::Blue => Color32::from_rgb(50, 100, 220),
        }
    }

    fn to_highlight_color(self, _dark_mode: bool) -> Color32 {
        match self {
            BallColor::Red => Color32::from_rgb(255, 100, 100),
            BallColor::Purple => Color32::from_rgb(200, 120, 255),
            BallColor::Green => Color32::from_rgb(120, 255, 140),
            BallColor::Brown => Color32::from_rgb(255, 180, 100),
            BallColor::LightBlue => Color32::from_rgb(140, 230, 255),
            BallColor::Blue => Color32::from_rgb(120, 170, 255),
        }
    }

    fn random(rng: &mut fastrand::Rng) -> BallColor {
        BallColor::all()[rng.usize(..6)]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Pos {
    x: usize,
    y: usize,
}

impl Pos {
    fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }

    fn neighbors(&self) -> [Pos; 4] {
        let x = self.x as isize;
        let y = self.y as isize;
        [
            Pos::new((x + 1) as usize, y as usize),
            Pos::new((x - 1) as usize, y as usize),
            Pos::new(x as usize, (y + 1) as usize),
            Pos::new(x as usize, (y - 1) as usize),
        ]
    }

    fn is_valid(&self) -> bool {
        self.x < BOARD_SIZE && self.y < BOARD_SIZE
    }
}

#[derive(Debug, Clone, Copy)]
struct Ball {
    color: BallColor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GameState {
    AppTurn,
    UserSelectBall,
    UserSelectTarget,
    GameOver,
}

struct RLinesApp {
    board: [[Option<Ball>; BOARD_SIZE]; BOARD_SIZE],
    preview_balls: [BallColor; 3],
    score: u32,
    state: GameState,
    selected_ball: Option<Pos>,
    dark_mode: bool,
    rng: fastrand::Rng,
    path_cache: Vec<Pos>,
}

impl Default for RLinesApp {
    fn default() -> Self {
        let mut rng = fastrand::Rng::new();
        let mut app = Self {
            board: [[None; BOARD_SIZE]; BOARD_SIZE],
            preview_balls: [
                BallColor::random(&mut rng),
                BallColor::random(&mut rng),
                BallColor::random(&mut rng),
            ],
            score: 0,
            state: GameState::AppTurn,
            selected_ball: None,
            dark_mode: true,
            rng,
            path_cache: Vec::new(),
        };
        app.app_turn();
        app
    }
}

impl RLinesApp {
    fn app_turn(&mut self) {
        let empty_positions = self.get_empty_positions();

        for &color in &self.preview_balls {
            if let Some(pos) = self.rng.choice(&empty_positions) {
                self.board[pos.y][pos.x] = Some(Ball { color });
            }
        }

        self.check_and_remove_lines();

        if self.get_empty_positions().is_empty() {
            self.state = GameState::GameOver;
            return;
        }

        self.preview_balls = [
            BallColor::random(&mut self.rng),
            BallColor::random(&mut self.rng),
            BallColor::random(&mut self.rng),
        ];

        self.state = GameState::UserSelectBall;
    }

    fn get_empty_positions(&self) -> Vec<Pos> {
        let mut positions = Vec::new();
        for y in 0..BOARD_SIZE {
            for x in 0..BOARD_SIZE {
                if self.board[y][x].is_none() {
                    positions.push(Pos::new(x, y));
                }
            }
        }
        positions
    }

    fn find_path(&self, from: Pos, to: Pos) -> Option<Vec<Pos>> {
        if from == to {
            return Some(vec![]);
        }
        if self.board[to.y][to.x].is_some() {
            return None;
        }

        let mut queue = VecDeque::new();
        let mut visited = [[false; BOARD_SIZE]; BOARD_SIZE];
        let mut parent = [[None; BOARD_SIZE]; BOARD_SIZE];

        queue.push_back(from);
        visited[from.y][from.x] = true;

        while let Some(current) = queue.pop_front() {
            if current == to {
                let mut path = Vec::new();
                let mut pos = to;
                while pos != from {
                    path.push(pos);
                    pos = parent[pos.y][pos.x].unwrap();
                }
                path.reverse();
                return Some(path);
            }

            for neighbor in current.neighbors() {
                if neighbor.is_valid() && !visited[neighbor.y][neighbor.x] {
                    if self.board[neighbor.y][neighbor.x].is_none() {
                        visited[neighbor.y][neighbor.x] = true;
                        parent[neighbor.y][neighbor.x] = Some(current);
                        queue.push_back(neighbor);
                    }
                }
            }
        }

        None
    }

    fn check_and_remove_lines(&mut self) -> bool {
        let mut to_remove = [[false; BOARD_SIZE]; BOARD_SIZE];
        let mut any_removed = false;

        for y in 0..BOARD_SIZE {
            for x in 0..BOARD_SIZE {
                if let Some(ball) = self.board[y][x] {
                    if !to_remove[y][x] {
                        let line = self.find_line(x, y, ball.color);
                        if line.len() >= 5 {
                            any_removed = true;
                            for pos in line {
                                to_remove[pos.y][pos.x] = true;
                            }
                        }
                    }
                }
            }
        }

        if any_removed {
            let mut removed_count = 0;
            for y in 0..BOARD_SIZE {
                for x in 0..BOARD_SIZE {
                    if to_remove[y][x] {
                        self.board[y][x] = None;
                        removed_count += 1;
                    }
                }
            }
            self.score += removed_count;
        }

        any_removed
    }

    fn find_line(&self, start_x: usize, start_y: usize, color: BallColor) -> Vec<Pos> {
        let directions = [
            (1, 0),
            (0, 1),
            (1, 1),
            (1, -1),
        ];

        let mut best_line = Vec::new();

        for (dx, dy) in directions {
            let mut line = Vec::new();
            let mut x = start_x as isize;
            let mut y = start_y as isize;

            while x >= 0 && x < BOARD_SIZE as isize && y >= 0 && y < BOARD_SIZE as isize {
                let ux = x as usize;
                let uy = y as usize;
                if let Some(ball) = self.board[uy][ux] {
                    if ball.color == color {
                        line.push(Pos::new(ux, uy));
                    } else {
                        break;
                    }
                } else {
                    break;
                }
                x += dx;
                y += dy;
            }

            x = start_x as isize - dx;
            y = start_y as isize - dy;

            while x >= 0 && x < BOARD_SIZE as isize && y >= 0 && y < BOARD_SIZE as isize {
                let ux = x as usize;
                let uy = y as usize;
                if let Some(ball) = self.board[uy][ux] {
                    if ball.color == color {
                        line.push(Pos::new(ux, uy));
                    } else {
                        break;
                    }
                } else {
                    break;
                }
                x -= dx;
                y -= dy;
            }

            if line.len() > best_line.len() {
                best_line = line;
            }
        }

        best_line
    }

    fn handle_tile_click(&mut self, pos: Pos) {
        match self.state {
            GameState::UserSelectBall => {
                if self.board[pos.y][pos.x].is_some() {
                    self.selected_ball = Some(pos);
                    self.state = GameState::UserSelectTarget;
                    self.path_cache = self.find_path(pos, pos).unwrap_or_default();
                }
            }
            GameState::UserSelectTarget => {
                if let Some(selected) = self.selected_ball {
                    if selected == pos {
                        self.selected_ball = None;
                        self.state = GameState::UserSelectBall;
                        self.path_cache.clear();
                    } else if self.board[pos.y][pos.x].is_none() {
                        if let Some(path) = self.find_path(selected, pos) {
                            self.path_cache = path.clone();
                            let ball = self.board[selected.y][selected.x].unwrap();
                            self.board[selected.y][selected.x] = None;
                            self.board[pos.y][pos.x] = Some(ball);
                            self.selected_ball = None;
                            self.path_cache.clear();

                            if !self.check_and_remove_lines() {
                                self.state = GameState::AppTurn;
                                self.app_turn();
                            } else {
                                self.state = GameState::UserSelectBall;
                            }
                        }
                    } else {
                        self.selected_ball = Some(pos);
                        self.path_cache = self.find_path(pos, pos).unwrap_or_default();
                    }
                }
            }
            _ => {}
        }
    }

    fn draw_board(&self, ui: &mut Ui, board_rect: Rect) {
        let (bg_color, grid_color) = if self.dark_mode {
            (Color32::BLACK, Color32::from_gray(60))
        } else {
            (Color32::WHITE, Color32::from_gray(200))
        };

        ui.painter().rect_filled(board_rect, 0.0, bg_color);

        for i in 0..=BOARD_SIZE {
            let x = board_rect.left() + i as f32 * TILE_SIZE;
            ui.painter().line_segment(
                [Pos2::new(x, board_rect.top()), Pos2::new(x, board_rect.bottom())],
                Stroke::new(1.0, grid_color),
            );
            let y = board_rect.top() + i as f32 * TILE_SIZE;
            ui.painter().line_segment(
                [Pos2::new(board_rect.left(), y), Pos2::new(board_rect.right(), y)],
                Stroke::new(1.0, grid_color),
            );
        }

        for y in 0..BOARD_SIZE {
            for x in 0..BOARD_SIZE {
                let center = Pos2::new(
                    board_rect.left() + x as f32 * TILE_SIZE + TILE_SIZE / 2.0,
                    board_rect.top() + y as f32 * TILE_SIZE + TILE_SIZE / 2.0,
                );

                if let Some(ball) = self.board[y][x] {
                    let is_selected = self.selected_ball == Some(Pos::new(x, y));
                    let color = if is_selected {
                        ball.color.to_highlight_color(self.dark_mode)
                    } else {
                        ball.color.to_color32(self.dark_mode)
                    };

                    ui.painter().circle_filled(center, BALL_RADIUS, color);
                    ui.painter().circle_stroke(center, BALL_RADIUS, Stroke::new(2.0, Color32::from_gray(100)));
                }
            }
        }

        if let Some(selected) = self.selected_ball {
            let center = Pos2::new(
                board_rect.left() + selected.x as f32 * TILE_SIZE + TILE_SIZE / 2.0,
                board_rect.top() + selected.y as f32 * TILE_SIZE + TILE_SIZE / 2.0,
            );
            ui.painter().circle_stroke(center, BALL_RADIUS + 3.0, Stroke::new(3.0, Color32::YELLOW));
        }
    }

    fn draw_preview(&self, ui: &mut Ui, rect: Rect) {
        let (bg_color, text_color) = if self.dark_mode {
            (Color32::BLACK, Color32::WHITE)
        } else {
            (Color32::WHITE, Color32::BLACK)
        };

        ui.painter().rect_filled(rect, 4.0, bg_color);

        let total_width = 3.0 * (PREVIEW_BALL_RADIUS * 2.0 + 10.0) - 10.0;
        let start_x = rect.center().x - total_width / 2.0;
        let y = rect.center().y;

        ui.painter().text(
            Pos2::new(rect.left() + 10.0, rect.center().y),
            egui::Align2::LEFT_CENTER,
            "Next:",
            egui::FontId::proportional(16.0),
            text_color,
        );

        for (i, &color) in self.preview_balls.iter().enumerate() {
            let x = start_x + i as f32 * (PREVIEW_BALL_RADIUS * 2.0 + 10.0) + PREVIEW_BALL_RADIUS;
            let center = Pos2::new(x, y);
            ui.painter().circle_filled(center, PREVIEW_BALL_RADIUS, color.to_color32(self.dark_mode));
            ui.painter().circle_stroke(center, PREVIEW_BALL_RADIUS, Stroke::new(2.0, Color32::from_gray(100)));
        }
    }
}

impl eframe::App for RLinesApp {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut Frame) {
        let (bg_color, text_color) = if self.dark_mode {
            (Color32::BLACK, Color32::WHITE)
        } else {
            (Color32::WHITE, Color32::BLACK)
        };

        ui.ctx().set_pixels_per_point(1.5);

        egui::CentralPanel::default().frame(egui::Frame::default().fill(bg_color)).show(ui, |ui: &mut Ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);

                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("RLines").size(32.0).color(text_color).strong());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(20.0);
                        ui.label(egui::RichText::new(format!("Score: {}", self.score)).size(24.0).color(text_color).strong());
                        ui.add_space(20.0);
                        if ui.button(if self.dark_mode { "☀ Light" } else { "🌙 Dark" }).clicked() {
                            self.dark_mode = !self.dark_mode;
                        }
                    });
                });

                ui.add_space(10.0);

                let preview_rect = Rect::from_min_size(
                    Pos2::new(ui.available_width() / 2.0 - 150.0, ui.next_widget_position().y),
                    Vec2::new(300.0, 60.0),
                );
                self.draw_preview(ui, preview_rect);
                ui.add_space(70.0);

                let board_size = BOARD_SIZE as f32 * TILE_SIZE;
                let board_rect = Rect::from_min_size(
                    Pos2::new(ui.available_width() / 2.0 - board_size / 2.0, ui.next_widget_position().y),
                    Vec2::new(board_size, board_size),
                );

                let response = ui.allocate_rect(board_rect, Sense::click());

                if response.clicked() {
                    if let Some(pos) = response.interact_pointer_pos() {
                        let rel_x = pos.x - board_rect.left();
                        let rel_y = pos.y - board_rect.top();
                        if rel_x >= 0.0 && rel_x < board_size && rel_y >= 0.0 && rel_y < board_size {
                            let x = (rel_x / TILE_SIZE) as usize;
                            let y = (rel_y / TILE_SIZE) as usize;
                            if x < BOARD_SIZE && y < BOARD_SIZE {
                                self.handle_tile_click(Pos::new(x, y));
                            }
                        }
                    }
                }

                self.draw_board(ui, board_rect);

                ui.add_space(20.0);

                match self.state {
                    GameState::AppTurn => {
                        ui.label(egui::RichText::new("Application's turn...").size(18.0).color(Color32::YELLOW));
                    }
                    GameState::UserSelectBall => {
                        ui.label(egui::RichText::new("Your turn: Select a ball to move").size(18.0).color(Color32::LIGHT_GREEN));
                    }
                    GameState::UserSelectTarget => {
                        ui.label(egui::RichText::new("Your turn: Select target tile").size(18.0).color(Color32::LIGHT_GREEN));
                    }
                    GameState::GameOver => {
                        ui.label(egui::RichText::new("Game Over!").size(24.0).color(Color32::RED).strong());
                        ui.label(egui::RichText::new(format!("Final Score: {}", self.score)).size(20.0).color(text_color));
                        if ui.button("New Game").clicked() {
                            *self = Self::default();
                        }
                    }
                }
            });
        });

        if self.state == GameState::AppTurn {
            ui.ctx().request_repaint();
            std::thread::sleep(std::time::Duration::from_millis(500));
            self.app_turn();
        }
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([600.0, 800.0])
            .with_title("RLines"),
        ..Default::default()
    };
    eframe::run_native("RLines", options, Box::new(|_cc| Ok(Box::new(RLinesApp::default()))))
}