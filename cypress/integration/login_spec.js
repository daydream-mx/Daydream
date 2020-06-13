describe('LoginPage', () => {

    beforeEach(function () {
        // We use cy.visit({onBeforeLoad: ...}) to stub
        // window.fetch before any app code runs
        cy.visit('/', {
            onBeforeLoad(win) {
                let fetch = win.fetch;
                cy.stub(win, 'fetch')
                    .callsFake((arg) => {
                        if (typeof arg === "string") {
                            console.log("REQUESTED: " + arg);
                            if (arg == 'daydream.wasm') {
                                return fetch(arg);
                            }
                        } else if (typeof arg === "object") {
                            console.log("REQUESTED: ", arg);
                            if (arg["url"] === "http://localhost:8448/_matrix/client/r0/login") {
                                console.log("handling login");
                                return new Promise((resolve, reject) => {
                                        const resp_data = {
                                            "user_id": "@carl:example.com",
                                            "access_token": "123456",
                                            "device_id": "KCZFUCGSLZ"
                                        };
                                        const resp = new Blob([JSON.stringify(resp_data, null, 2)], {type: 'application/json'});

                                        const init = {
                                            "status": 200,
                                            "statusText": "Ok",
                                            headers: {'Content-type': 'application/json'}
                                        };
                                        const response = new win.Response(resp, init);
                                        Object.defineProperty(response, "url", {value: arg["url"]});
                                        resolve(response)
                                    }
                                )
                            } else {
                                console.log("handling sync");
                                return new Promise((resolve, reject) => {
                                        const resp_data = {
                                            "next_batch": "s72595_4483_1934",
                                            "rooms": {
                                                "join": {
                                                    "!726s6s6q:example.com": {
                                                        "unread_notifications": {
                                                            "highlight_count": 0,
                                                            "notification_count": 0,
                                                        },
                                                        "summary": {
                                                            "m.heroes": [
                                                                "@alice:example.com",
                                                                "@bob:example.com"
                                                            ],
                                                            "m.joined_member_count": 2,
                                                            "m.invited_member_count": 0
                                                        },
                                                        "state": {
                                                            "events": [
                                                                {
                                                                    "content": {
                                                                        "membership": "join",
                                                                        "avatar_url": "mxc://example.org/SEsfnsuifSDFSSEF",
                                                                        "displayname": "Alice Margatroid"
                                                                    },
                                                                    "type": "m.room.member",
                                                                    "event_id": "$143273582443PhrSn:example.org",
                                                                    "room_id": "!726s6s6q:example.com",
                                                                    "sender": "@example:example.org",
                                                                    "origin_server_ts": 1432735824653,
                                                                    "unsigned": {
                                                                        "age": 1234
                                                                    },
                                                                    "state_key": "@alice:example.org"
                                                                }
                                                            ]
                                                        },
                                                        "timeline": {
                                                            "events": [
                                                                {
                                                                    "content": {
                                                                        "membership": "join",
                                                                        "avatar_url": "mxc://example.org/SEsfnsuifSDFSSEF",
                                                                        "displayname": "Alice Margatroid"
                                                                    },
                                                                    "type": "m.room.member",
                                                                    "event_id": "$143273582443PhrSn:example.org",
                                                                    "room_id": "!726s6s6q:example.com",
                                                                    "sender": "@example:example.org",
                                                                    "origin_server_ts": 1432735824653,
                                                                    "unsigned": {
                                                                        "age": 1234
                                                                    },
                                                                    "state_key": "@alice:example.org"
                                                                },
                                                                {
                                                                    "content": {
                                                                        "body": "This is an example text message",
                                                                        "msgtype": "m.text",
                                                                        "format": "org.matrix.custom.html",
                                                                        "formatted_body": "<b>This is an example text message</b>"
                                                                    },
                                                                    "type": "m.room.message",
                                                                    "event_id": "$143273582443PhrSn:example.org",
                                                                    "room_id": "!726s6s6q:example.com",
                                                                    "sender": "@example:example.org",
                                                                    "origin_server_ts": 1432735824653,
                                                                    "unsigned": {
                                                                        "age": 1234
                                                                    }
                                                                }
                                                            ],
                                                            "limited": true,
                                                            "prev_batch": "t34-23535_0_0"
                                                        },
                                                        "ephemeral": {
                                                            "events": [
                                                                {
                                                                    "content": {
                                                                        "user_ids": [
                                                                            "@alice:matrix.org",
                                                                            "@bob:example.com"
                                                                        ]
                                                                    },
                                                                    "type": "m.typing",
                                                                    "room_id": "!jEsUZKDJdhlrceRyVU:example.org"
                                                                }
                                                            ]
                                                        },
                                                        "account_data": {
                                                            "events": [
                                                                {
                                                                    "content": {
                                                                        "tags": {
                                                                            "u.work": {
                                                                                "order": 0.9
                                                                            }
                                                                        }
                                                                    },
                                                                    "type": "m.tag"
                                                                },
                                                                {
                                                                    "type": "org.example.custom.room.config",
                                                                    "content": {
                                                                        "custom_config_key": "custom_config_value"
                                                                    }
                                                                }
                                                            ]
                                                        }
                                                    }
                                                },
                                                "invite": {
                                                    "!696r7674:example.com": {
                                                        "invite_state": {
                                                            "events": [
                                                                {
                                                                    "sender": "@alice:example.com",
                                                                    "type": "m.room.name",
                                                                    "state_key": "",
                                                                    "content": {
                                                                        "name": "My Room Name"
                                                                    }
                                                                },
                                                                {
                                                                    "sender": "@alice:example.com",
                                                                    "type": "m.room.member",
                                                                    "state_key": "@bob:example.com",
                                                                    "content": {
                                                                        "membership": "invite"
                                                                    }
                                                                }
                                                            ]
                                                        }
                                                    }
                                                },
                                                "leave": {}
                                            }
                                        };
                                        const resp = new Blob([JSON.stringify(resp_data, null, 2)], {type: 'application/json'});

                                        const init = {
                                            "status": 200,
                                            "statusText": "Ok",
                                            headers: {'Content-type': 'application/json'}
                                        };
                                        const response = new win.Response(resp, init);
                                        Object.defineProperty(response, "url", {value: arg["url"]});
                                        resolve(response)
                                    }
                                )
                            }
                        }

                    });
            },
        })
    })

    /* beforeEach(function () {
         cy.server();

         cy.fixture('login.json').as('loginJSON')
         cy.route('POST', '/_matrix/client/r0/login', '@loginJSON')
         cy.route({
             method: 'OPTIONS',      // Route all OPTION requests
             url: '/_matrix/*',
             response: {}
         });

         // We use cy.visit({onBeforeLoad: ...}) to spy on
         // window.fetch before any app code runs
         cy.visit('/')

     })*/
    it('Does login', () => {
        cy.get('form#login_form').within(() => {
            // Set homeserverurl
            const homeserver_url = "http://localhost:8448";
            cy.get('input#homeserver').type(homeserver_url);
            cy.get('input#homeserver').should('have.value', homeserver_url);

            // Set username
            const username = "@carl:example.com";
            cy.get('input#username').type(username);
            cy.get('input#username').should('have.value', username);

            // Set password
            const password = "12345";
            cy.get('input#password').type(username);
            cy.get('input#password').should('have.value', username);
        });
        cy.get('form#login_form').submit();
        cy.get('ul.scrollable').should('be.visible');
    })
})
