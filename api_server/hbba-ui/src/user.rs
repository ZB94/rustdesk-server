use crate::Ui;
use crossbeam_channel::{unbounded, Receiver, Sender};
use eframe::egui::{Align2, Color32, Grid, Label, Layout, RichText, Sense, TextEdit, Window};
use eframe::emath::Align;
use reqwasm::http::Method;
use serde::Serialize;

pub struct User {
    username: String,
    password: String,
    pwd_current: String,
    pwd_new1: String,
    pwd_new2: String,
    perm: Permission,
    remembered: bool,
    access_token: Option<String>,
    chanel: (Sender<(u16, Response)>, Receiver<(u16, Response)>),
    message: Vec<(Color32, String)>,
}

impl User {
    pub fn new() -> Self {
        Self {
            username: "".to_string(),
            password: "".to_string(),
            pwd_current: "".to_string(),
            pwd_new1: "".to_string(),
            pwd_new2: "".to_string(),
            perm: Permission::User,
            remembered: true,
            access_token: None,
            chanel: unbounded(),
            message: Vec::with_capacity(5),
        }
    }

    #[inline]
    pub fn login(&self) -> bool {
        self.access_token.is_some()
    }

    pub fn ui(&mut self, ui: &mut Ui) {
        if self.login() {
            self.ui_user(ui);
        } else {
            self.ui_login(ui);
        }
        self.ui_message(ui);
    }

    fn ui_user(&mut self, ui: &mut Ui) {
        self.show_user_info(ui);
    }

    fn show_user_info(&mut self, ui: &mut Ui) {
        Window::new("用户信息")
            .resizable(false)
            .show(ui.ctx(), |ui| {
                Grid::new("user info").spacing([4.0, 8.0]).show(ui, |ui| {
                    ui.label("用户名");
                    ui.label(&self.username);
                    ui.end_row();

                    ui.label("权限");
                    ui.label(self.perm.name());
                    ui.end_row();

                    ui.label("当前密码");
                    ui.add(
                        TextEdit::singleline(&mut self.pwd_current)
                            .password(true)
                            .hint_text("请输入当前密码"),
                    );
                    ui.end_row();

                    ui.label("修改密码");
                    ui.add(
                        TextEdit::singleline(&mut self.pwd_new1)
                            .password(true)
                            .hint_text("请输入新密码"),
                    );
                    ui.end_row();

                    ui.label("确认密码");
                    ui.add(
                        TextEdit::singleline(&mut self.pwd_new2)
                            .password(true)
                            .hint_text("与修改密码相同"),
                    );
                    ui.end_row();

                    ui.label("");
                    ui.with_layout(Layout::right_to_left(), |ui| {
                        if ui
                            .button(RichText::new("退出").color(Color32::RED))
                            .clicked()
                        {
                            self.access_token = None;
                        }
                        if ui.button("修改密码").clicked() {
                            if self.pwd_current.is_empty()
                                || self.pwd_new1.is_empty()
                                || self.pwd_new2.is_empty()
                            {
                                self.add_error("当前密码、修改密码、确认密码均不能为空");
                                return;
                            }

                            if self.pwd_new1 != self.pwd_new2 {
                                self.add_error("修改密码与确认密码不相同");
                                return;
                            }

                            self.req(
                                Method::POST,
                                "/manage/change_password",
                                Some(json!({
                                    "old_password": self.pwd_current.clone(),
                                    "new_password": self.pwd_new1.clone()
                                })),
                            );
                        }
                    });
                    ui.end_row();
                });
            });
    }

    fn ui_login(&mut self, ui: &mut Ui) {
        Window::new("登录")
            .anchor(Align2::CENTER_TOP, [0.0, 200.0])
            .collapsible(false)
            .resizable(false)
            .show(ui.ctx(), |ui| {
                Grid::new("login").spacing([4.0, 8.0]).show(ui, |ui| {
                    ui.label("用户名");
                    ui.text_edit_singleline(&mut self.username);
                    ui.end_row();

                    ui.label("密码");
                    ui.add(TextEdit::singleline(&mut self.password).password(true));
                    ui.end_row();

                    ui.label("用户类型");
                    ui.horizontal(|ui| {
                        ui.selectable_value(
                            &mut self.perm,
                            Permission::User,
                            Permission::User.name(),
                        );
                        ui.selectable_value(
                            &mut self.perm,
                            Permission::Admin,
                            Permission::Admin.name(),
                        );
                    });
                    ui.end_row();

                    ui.checkbox(&mut self.remembered, "记住密码")
                        .on_hover_text("仅在当前页面有效。刷新或关闭页面后重置");
                    ui.with_layout(Layout::right_to_left(), |ui| {
                        if ui.button("登录").clicked() {
                            if self.username.is_empty() || self.password.is_empty() {
                                self.add_error("用户名或密码不能为空");
                                return;
                            }

                            let user = json!({
                                "username": self.username.clone(),
                                "password": self.password.clone(),
                                "perm": self.perm
                            });

                            self.req(Method::POST, "/manage/login", Some(user));
                        }
                    });
                    ui.end_row();
                })
            });
    }

    fn ui_message(&mut self, ui: &mut Ui) {
        while let Ok((code, resp)) = self.chanel.1.try_recv() {
            if code == 401 {
                self.access_token = None;
                self.add_error("当前用户授权已过期或无效，请重新登录");
                while let Ok(_) = self.chanel.1.try_recv() {}
                return;
            }

            if let Some(err) = resp.error {
                self.add_error(err);
            }

            match resp.data {
                Some(ResponseBody::Login { access_token }) => {
                    if !self.remembered {
                        self.password.clear();
                    }
                    self.access_token = Some(access_token);
                }
                Some(ResponseBody::Empty {}) => {
                    self.add_info("操作成功");
                }
                _ => {}
            }
        }

        ui.with_layout(Layout::bottom_up(Align::Center), |ui| {
            let remove = self
                .message
                .iter()
                .enumerate()
                .rev()
                .filter_map(|(idx, (color, msg))| {
                    let text = RichText::new(msg).color(*color);
                    let label = Label::new(text).sense(Sense::click());
                    ui.add(label).clicked().then(|| idx)
                })
                .collect::<Vec<_>>();
            for idx in remove {
                self.message.remove(idx);
            }
        });
    }

    fn req<T>(&self, method: Method, url: &'static str, data: Option<T>)
    where
        T: Serialize + 'static,
    {
        let sender = self.chanel.0.clone();
        crate::utils::request(
            method,
            url,
            data,
            self.access_token.clone(),
            move |resp| match resp {
                Ok(d) => {
                    let _ = sender.send(d);
                }
                Err(e) => {
                    let _ = sender.send((0, Response::error(format!("请求出错: {}", e))));
                }
            },
        )
    }

    #[inline]
    fn add_error<S: ToString>(&mut self, error: S) {
        self.add_message(error, Color32::RED);
    }

    #[inline]
    fn add_info<S: ToString>(&mut self, info: S) {
        self.add_message(info, Color32::GREEN);
    }

    fn add_message<S: ToString, C: Into<Color32>>(&mut self, message: S, color: C) {
        if self.message.len() == self.message.capacity() {
            self.message.remove(0);
        }
        self.message.push((color.into(), message.to_string()));
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Hash, Copy, Clone)]
pub enum Permission {
    User,
    Admin,
}

impl Permission {
    pub fn name(&self) -> &'static str {
        match self {
            Self::User => "普通用户",
            Self::Admin => "管理员",
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Response {
    pub error: Option<String>,
    #[serde(flatten)]
    pub data: Option<ResponseBody>,
}

impl Response {
    pub fn error<S: ToString>(error: S) -> Self {
        Self {
            error: Some(error.to_string()),
            data: None,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ResponseBody {
    Login { access_token: String },
    Empty {},
}
