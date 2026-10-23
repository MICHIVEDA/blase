use std::error::Error;
use std::time::Duration;

use atspi::proxy::accessible::ObjectRefExt;
use atspi::proxy::component::ComponentProxy;
use atspi::{AccessibilityConnection, CoordType, ObjectRefOwned, Role, State};


#[derive(Debug)]
pub struct Target {
    pub role: Role,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub object: ObjectRefOwned,
}

fn is_clickable(role: Role) -> bool {
    matches!(
	role,
	Role::Link
	    | Role::Button
	    | Role::ToggleButton
    )
}

pub async fn get_active_window(conn: &AccessibilityConnection) -> Result<Option<ObjectRefOwned>, Box<dyn Error>> {
    let atspi = conn.connection();

    let root = conn.root_accessible_on_registry().await?;


    for app in root.get_children().await?.iter() {
	let Ok(app_proxy) = app.as_accessible_proxy(atspi).await else {
	    continue;
	};

	let Ok(frames) = app_proxy.get_children().await else {
	    continue;
	};

	for frame in frames.iter() {
	    let Ok(frame_proxy) = frame.as_accessible_proxy(atspi).await else{
		continue;
	    };

	    let Ok(states) = frame_proxy.get_state().await else {
		continue;
	    };

	    if states.contains(State::Active) {
		println!(
		    "active window: {} in application {}", frame_proxy.name().await?, app_proxy.name().await?,
		);

		return Ok(Some(frame.clone()));
	    }
	}
    }
    Ok(None)
}

async fn extract_info(conn: &AccessibilityConnection, node: &ObjectRefOwned) -> Result<(i32, i32, i32, i32), Box<dyn Error>> {
    let name = node.name_as_str().ok_or("object reference has no bus name")?;

    let component = ComponentProxy::builder(conn.connection())
        .destination(name)?
	.path(node.path_as_str())?
	.build()
	.await?;

    Ok(component.get_extents(CoordType::Screen).await?)
}

pub async fn collect_targets(conn: &AccessibilityConnection, frame: &ObjectRefOwned) -> Result<Vec<Target>, Box<dyn Error>>{
    let atspi = conn.connection();

    let mut targets: Vec<Target> = Vec::new();

    let mut stack: Vec<ObjectRefOwned> = vec![frame.clone()];

    while let Some(node) = stack.pop() {

	let Ok(proxy) = node.as_accessible_proxy(atspi).await else {
	    continue;
	};

	let Ok(states) = proxy.get_state().await else {
	    continue;
	};

	if !states.contains(State::Showing) { continue; }

	let Ok(role) = proxy.get_role().await else {
	    continue;
	};

	if is_clickable(role) {
	    if let Ok((x,y,width,height)) = extract_info(conn, &node).await {
		if width > 0 && height > 0 {
		    targets.push(Target {
			role,
			x,
			y,
			width,
			height,
			object: node.clone(),
		    });
		}
	    }
	}

	if let Ok(children) = proxy.get_children().await {
	    stack.extend(children);
	}
    }
    Ok(targets)
}


async fn scan() -> Result<(), Box<dyn Error>>{
    let conn = atspi::AccessibilityConnection::new().await?;

    loop {
	match get_active_window(&conn).await {
	    Ok(Some(frame)) => match collect_targets(&conn, &frame).await {
		Ok(targets) => {
		    println!("{} targets", targets.len());
		    for target in &targets {
			println!("{:?} at ({}, {}), size {}x{}", target.role, target.x, target.y, target.width, target.height);
		    }
		},
		Err(e) => eprintln!("collecting targets failed {e}"),
	    }
	    Ok(None) => println!("no active frame"),
	    Err(e) => eprintln!("finding active window failed {e}"),
	}
	tokio::time::sleep(Duration::from_secs(1)).await;
	}
}


pub fn run() -> Result<(), Box<dyn Error>>{
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    runtime.block_on(scan())
    
}
