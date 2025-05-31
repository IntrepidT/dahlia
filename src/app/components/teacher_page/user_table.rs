use crate::app::models::user::User;
use leptos::*;
use std::rc::Rc;

const TABLE_CONTAINER_STYLE: &str =
    "bg-[#F9F9F8] rounded-lg shadow-sm border border-[#DADADA] overflow-hidden";
const TABLE_HEADER_STYLE: &str =
    "py-5 px-6 flex justify-between items-center border-b border-[#2E3A59] bg-[#2E3A59]";
const TABLE_WRAPPER_STYLE: &str = "overflow-x-auto h-[34rem]";
const TABLE_STYLE: &str = "min-w-full divide-y divide-[#DADADA]";
const HEADER_CELL_STYLE: &str =
    "px-6 py-3 text-left text-sm font-medium text-[#2E3A59] uppercase tracking-wider";
const CELL_STYLE: &str = "px-6 py-4 whitespace-nowrap text-sm bg-[#F9F9F8]";
const SELECTED_ROW_STYLE: &str =
    "bg-[#DADADA] border-l-4 border-r-2 border-t-2 border-b-2 border-[#2E3A59]";

#[component]
pub fn UserTable(
    #[prop(into)] users: Resource<i32, Option<Vec<User>>>,
    #[prop(into)] search_term: Signal<String>,
    #[prop(into)] is_panel_expanded: Signal<bool>,
) -> impl IntoView {
    let filtered_users = create_memo(move |_| {
        let search = search_term().trim().to_lowercase();

        users
            .get()
            .unwrap_or(None)
            .unwrap_or_default()
            .into_iter()
            .filter(|user| {
                let matches_search = search.is_empty()
                    || user
                        .first_name
                        .as_ref()
                        .map(|name| name.to_lowercase().contains(&search))
                        .unwrap_or(false)
                    || user
                        .last_name
                        .as_ref()
                        .map(|name| name.to_lowercase().contains(&search))
                        .unwrap_or(false)
                    || user.email.to_lowercase().contains(&search)
                    || user.username.to_lowercase().contains(&search)
                    || user.role.to_string().to_lowercase().contains(&search)
                    || user
                        .account_status
                        .to_string()
                        .to_lowercase()
                        .contains(&search);

                matches_search
            })
            .collect::<Vec<_>>()
    });

    let container_class = create_memo(move |_| {
        format!(
            "{} transition-all duration-300 ease-in-out",
            TABLE_CONTAINER_STYLE
        )
    });

    view! {
        <div class=move || container_class()>
            <div class=TABLE_HEADER_STYLE>
                <h2 class="text-xl font-medium text-white">
                    "Users"
                </h2>
                <span class="text-sm text-white">
                    {move || {
                        let count = filtered_users().len();
                        format!("{} {}", count, if count == 1 {"user"} else {"users"})
                    }}
                </span>
            </div>
            <div class=TABLE_WRAPPER_STYLE>
                <table class=TABLE_STYLE>
                    <thead class="bg-[#DADADA] sticky top-0 z-10">
                        <tr>
                            <th class=HEADER_CELL_STYLE>"Username"</th>
                            <th class=HEADER_CELL_STYLE>"First Name"</th>
                            <th class=HEADER_CELL_STYLE>"Last Name"</th>
                            <th class=HEADER_CELL_STYLE>"Email"</th>
                            <th class=HEADER_CELL_STYLE>"Phone"</th>
                            <th class=HEADER_CELL_STYLE>"Account Status"</th>
                            <th class=HEADER_CELL_STYLE>"Role"</th>
                        </tr>
                    </thead>
                    <Suspense fallback=move || view! {
                        <tr>
                            <td colspan="7" class="text-center p-8">
                                <div class="inline-block h-6 w-6 animate-spin rounded-full border-2 border-[#DADADA] border-t-[#2E3A59]"></div>
                            </td>
                        </tr>
                    }>
                        <tbody>
                            {move || {
                                let users = filtered_users();
                                if users.is_empty() {
                                    view! {
                                        <tr>
                                            <td colspan="7" class="px-6 py-12 text-center text-sm text-gray-500">
                                                "No users match your search criteria"
                                            </td>
                                        </tr>
                                    }.into_view()
                                } else {
                                    users.into_iter().map(|user| {
                                        // Clone all the necessary fields to avoid lifetime issues
                                        let username = user.username.clone();
                                        let first_name = user.first_name.clone().unwrap_or_default();
                                        let last_name = user.last_name.clone().unwrap_or_default();
                                        let email = user.email.clone();
                                        let phone = user.phone_number.clone().unwrap_or_default();
                                        let status = user.account_status.to_string();
                                        let role = user.role.clone();

                                        view! {
                                            <tr class="hover:bg-[#DADADA] hover:bg-opacity-70 cursor-pointer border-b border-[#DADADA]">
                                                <td class=format!("{} {}", CELL_STYLE, "text-[#2E3A59]")>{username}</td>
                                                <td class=format!("{} {}", CELL_STYLE, "font-medium text-[#2E3A59]")>{first_name}</td>
                                                <td class=format!("{} {}", CELL_STYLE, "font-medium text-[#2E3A59]")>{last_name}</td>
                                                <td class=format!("{} {}", CELL_STYLE, "font-medium text-[#2E3A59]")>{email}</td>
                                                <td class=format!("{} {}", CELL_STYLE, "font-medium text-[#2E3A59]")>{phone}</td>
                                                <td class=format!("{} {}", CELL_STYLE, "font-medium text-[#2E3A59]")>{status}</td>
                                                <td class=format!("{} {}", CELL_STYLE, "font-medium text-[#2E3A59]")>{role.to_string()}</td>
                                            </tr>
                                        }
                                    }).collect_view()
                                }
                            }}
                        </tbody>
                    </Suspense>
                </table>
            </div>
        </div>
    }
}
