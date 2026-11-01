function getToken() {
    return sessionStorage.getItem("login_token");
}

function checkLogin() {
    if (!getToken()) {
        location.replace("/login");
    }
}