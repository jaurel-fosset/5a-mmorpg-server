use bevy::app::MainScheduleOrder;
use bevy::ecs::schedule::{ExecutorKind, ScheduleLabel};
use bevy::prelude::*;

pub struct ScheduleFactory<'a, T: ScheduleLabel>
{
    app: &'a mut App,
    label: T,
}

impl<'a, T: ScheduleLabel + Copy> ScheduleFactory<'a, T>
{
    pub fn register(app: &'a mut App, label: T) -> Self
    {
        let mut schedule = Schedule::new(label);
        if cfg!(target_arch = "wasm32")
        {
            schedule.set_executor_kind(ExecutorKind::SingleThreaded);
        }
        else
        {
            schedule.set_executor_kind(ExecutorKind::MultiThreaded);
        }
        app.add_schedule(schedule);

        Self { app, label }
    }
}

impl<'a, T: ScheduleLabel + Clone> ScheduleFactory<'a, T>
{
    pub fn register_clone(app: &'a mut App, label: T) -> Self
    {
        let mut schedule = Schedule::new(label.clone());
        if cfg!(target_arch = "wasm32")
        {
            schedule.set_executor_kind(ExecutorKind::SingleThreaded);
        } else {
            schedule.set_executor_kind(ExecutorKind::MultiThreaded);
        }
        app.add_schedule(schedule);

        Self { app, label }
    }
}

impl<'a, T: ScheduleLabel> ScheduleFactory<'a, T>
{
    pub fn after(self, schedule: impl ScheduleLabel)
    {
        let ScheduleFactory { app, label } = self;
        schedule_order(app).insert_after(schedule, label);
    }

    pub fn before(self, schedule: impl ScheduleLabel)
    {
        let ScheduleFactory { app, label } = self;
        schedule_order(app).insert_before(schedule, label);
    }

    pub fn after_startup(self, schedule: impl ScheduleLabel)
    {
        let ScheduleFactory { app, label } = self;
        schedule_order(app).insert_startup_after(schedule, label);
    }

    pub fn before_startup(self, schedule: impl ScheduleLabel)
    {
        let ScheduleFactory { app, label } = self;
        schedule_order(app).insert_startup_before(schedule, label);
    }
}

fn schedule_order(app: &'_ mut App) -> Mut<'_, MainScheduleOrder>
{
    app.world_mut().resource_mut()
}
