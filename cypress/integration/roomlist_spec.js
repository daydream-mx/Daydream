import {fake_matrix_api_handler} from "./matrix-fake-api";
import {login} from "./utils";

describe('RoomList', () => {
    const name_of_user = "Alice Margatroid";
    beforeEach(function () {
        // We use cy.visit({onBeforeLoad: ...}) to stub
        // window.fetch before any app code runs
        cy.visit('/', {
            onBeforeLoad(win) {
                let fetch = win.fetch;
                cy.stub(win, 'fetch')
                    .callsFake(args => fake_matrix_api_handler(args, fetch, win));
            },
        })
    });

    it("does allow clicking a room", () => {
        login();
        cy.get('ul.scrollable').contains(name_of_user).click();

        cy.log('check if the room title is shown after click');
        cy.get('h1').contains(name_of_user).should('be.visible');
    });

    it("should allow searching", () => {
        login();

        cy.log('check if the room is not shown if anything else is in the search');
        cy.get('input.uk-search-input').clear().type("blub");
        cy.get("ul.scrollable").contains(name_of_user).should("not.exist");

        cy.log('check if the room is shown if a part of the name is in the search');
        cy.get('input.uk-search-input').clear().type("Alice");
        cy.get("ul.scrollable").contains(name_of_user).should("exist").should("be.visible");

        cy.log('check if the room is shown if the full name is in the search');
        cy.get('input.uk-search-input').clear().type(name_of_user);
        cy.get("ul.scrollable").contains(name_of_user).should("exist").should("be.visible");
    });
})
