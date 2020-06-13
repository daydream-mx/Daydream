const login = () => {
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
}

export {login}
