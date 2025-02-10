
self.oninstall = event => {
    console.log('Service worker installed!', event);
};

self.onpush = async event => {
    const data = await event.data.json();
    console.log('Service worker received pushed data:', data);
    self.registration.showNotification(
        data.notification.title, data.notification.options);
    navigator.setAppBadge(data.appBadge);
};