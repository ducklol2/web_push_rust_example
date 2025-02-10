const publicKey = await(await fetch('public_key.txt')).text();
console.log('Fetched public push key from server:', publicKey);

const notifyBtn = document.querySelector('#notify');
function updateNotify() {
    switch (Notification.permission) {
        case 'granted':
            notifyBtn.disabled = false;
            notifyBtn.innerHTML = "Create (local) notification";
            notifyBtn.onclick = () => {
                new Notification("A local notification", {
                    body: 'This notification was created by local JavaScript.',
                    icon: 'guy.png'
                });
            };
            break;

        case 'denied':
            notifyBtn.disabled = true;
            notifyBtn.innerHTML = "Notifications disabled; permission must be updated or reset";
            break;

        default:
            notifyBtn.disabled = false;
            notifyBtn.innerHTML = "1. Grant notification permission";
            notifyBtn.onclick = () => {
                Notification.requestPermission().then(updateNotify);
            };
    }
}
updateNotify();

const workerBtn = document.querySelector('#worker');
async function updateWorker() {
    const registration = await navigator.serviceWorker.getRegistration();
    if (!registration) {
        workerBtn.innerHTML = "2. Register service worker";
        workerBtn.onclick = async () => {
            await navigator.serviceWorker.register("/worker.js", {
                scope: "/",
            });
            updateWorker();
            updatePush();
        };
    } else {
        workerBtn.innerHTML = "Unregister service worker";
        workerBtn.onclick = async () => {
            await registration.unregister();
            updateWorker();
            updatePush();
        };
    }
}
updateWorker();

const pushBtn = document.querySelector('#push');
async function updatePush() {
    const registration = await navigator.serviceWorker.getRegistration();
    if (!registration) {
        pushBtn.disabled = true;
        pushBtn.innerHTML = "Register service worker first";
        return;
    }

    const subscription = await registration.pushManager.getSubscription();
    if (!subscription) {
        pushBtn.disabled = false;
        pushBtn.innerHTML = "3. Register browser for subscription";
        pushBtn.onclick = async () => {
            await registration.pushManager.subscribe({
                userVisibleOnly: true,
                applicationServerKey: publicKey
            });
            updatePush();
        }
    } else {
        pushBtn.disabled = true;
        pushBtn.innerHTML = "Browser registered for subscription";
        console.log('Browser subscription data:', JSON.stringify(subscription));
        postSubscription(subscription);
    }
}
updatePush();

async function postSubscription(subscription) {
    const res = await fetch('/subscribe', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(subscription)
    });
    if (!res.ok) {
        console.error('/subscribe failed: ', res);
        return;
    }
};

const triggerPushBtn = document.querySelector('#triggerPush');
async function updateTriggerPush() {
    const registration = await navigator.serviceWorker.getRegistration();
    if (!registration) {
        triggerPushBtn.disabled = true;
        triggerPushBtn.innerHTML = "Register service worker first";
        return;
    }

    const subscription = await registration.pushManager.getSubscription();
    if (!subscription) {
        triggerPushBtn.disabled = true;
        triggerPushBtn.innerHTML = "Register browser for subscription first";
    }

    triggerPushBtn.disabled = false;
    triggerPushBtn.innerHTML = "4. Trigger a pushed notification from the server";
    triggerPushBtn.onclick = triggerPush;
}
updateTriggerPush();

async function triggerPush() {
    const res = await fetch('/push', {method: 'POST'});
    if (!res.ok) {
        console.error('/push failed: ', res);
        return;
    }
};

const refreshBtn = document.querySelector('#refresh');
refreshBtn.onclick = () => location.reload();